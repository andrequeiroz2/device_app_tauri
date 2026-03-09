use sqlx::Pool;
use sqlx::Sqlite;
use tracing::{error, info, instrument};

use crate::api::auth::auth_validator::validate_bearer;
use crate::api::icon::icon_model::{
    parse_icon_category, IconCreateDB, IconCreateInput, IconDeleteInput, IconListParams,
    IconListResponse, IconPublic, IconUpdateDB, IconUpdateInput,
};
use crate::api::icon::icon_query::{
    icon_get_by_uuid_query, icon_insert_query, icon_list_query, icon_soft_delete_query,
    icon_update_query,
};
use crate::api::icon::icon_tool::compose_icon_code;
use crate::api::model::{ApiError, ApiResponse};
use crate::api::user::user_query::user_get_by_uuid_query;
use uuid::Uuid;

#[instrument(skip(token, input, pool), fields(name = %input.name, iconify_id = %input.iconify_id))]
pub async fn create_icon_handler(
    token: &str,
    input: &IconCreateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<IconPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("create_icon_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let (name, iconify_id, category, color_hex) = input.validate().map_err(ApiError::err)?;

    let code = compose_icon_code(&iconify_id);
    let uuid = Uuid::new_v4().to_string();

    let db = IconCreateDB {
        uuid: uuid.clone(),
        code,
        name,
        iconify_id,
        category: category.as_str().to_string(),
        color: color_hex,
    };

    let icon = icon_insert_query(&db, pool).await.map_err(ApiError::err)?;

    let public = IconPublic::from(icon);

    info!(uuid = %public.uuid, code = %public.code, "create_icon_handler: icon created");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, params, pool), fields(category = ?params.category, page = ?params.page))]
pub async fn list_icons_handler(
    token: &str,
    params: &IconListParams,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<IconListResponse>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("list_icons_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let (icons, total) = icon_list_query(params, pool).await.map_err(ApiError::err)?;
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);

    let items: Vec<IconPublic> = icons.into_iter().map(IconPublic::from).collect();
    let resp = IconListResponse {
        items,
        total,
        page,
        page_size,
    };

    Ok(ApiResponse::ok(resp))
}

#[instrument(skip(token, pool), fields(icon_uuid = %icon_uuid))]
pub async fn get_icon_handler(
    token: &str,
    icon_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<IconPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_icon_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let icon = icon_get_by_uuid_query(icon_uuid, pool).await.map_err(ApiError::err)?;

    let public = IconPublic::from(icon);

    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, input, pool), fields(icon_uuid = %input.uuid))]
pub async fn update_icon_handler(
    token: &str,
    input: &IconUpdateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<IconPublic>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("update_icon_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    if input.uuid.trim().is_empty() {
        return Err(ApiError::err("UUID cannot be empty".to_string()));
    }

    input.validate_name().map_err(ApiError::err)?;
    input.validate_iconify_id().map_err(ApiError::err)?;
    input.validate_category().map_err(ApiError::err)?;

    let color_hex = input.validate_color().map_err(ApiError::err)?;

    let mut update_db = IconUpdateDB::default();

    if input.name.is_some() {
        update_db.name = input.name.as_ref().map(|s| s.trim().to_string());
    }
    if input.iconify_id.is_some() {
        let iconify_id = input.iconify_id.as_ref().map(|s| s.trim().to_string());
        update_db.iconify_id = iconify_id.clone();
        update_db.code = iconify_id.map(|i| compose_icon_code(&i));
    }
    if input.category.is_some() {
        let cat = input
            .category
            .as_ref()
            .and_then(|c| parse_icon_category(c).ok())
            .map(|ct| ct.as_str().to_string());
        update_db.category = cat;
    }
    if color_hex.is_some() {
        update_db.color = color_hex;
    }
    if input.is_active.is_some() {
        update_db.is_active = input.is_active;
    }

    if update_db.name.is_none()
        && update_db.iconify_id.is_none()
        && update_db.category.is_none()
        && update_db.color.is_none()
        && update_db.is_active.is_none()
    {
        return Err(ApiError::err("No fields to update".to_string()));
    }

    // Ao reativar (is_active=true), verificar se outro ícone ativo já usa o mesmo code
    if update_db.is_active == Some(true) {
        let existing = icon_get_by_uuid_query(&input.uuid, pool)
            .await
            .map_err(ApiError::err)?;
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM icons WHERE code = ?1 AND is_active = 1 AND uuid != ?2",
        )
        .bind(&existing.code)
        .bind(&input.uuid)
        .fetch_one(pool)
        .await
        .map_err(|e| ApiError::err(e.to_string()))?;
        if count > 0 {
            return Err(ApiError::err(format!(
                "Cannot reactivate: another active icon already uses code '{}'",
                existing.code
            )));
        }
    }

    let icon = icon_update_query(&input.uuid, &update_db, pool)
        .await
        .map_err(ApiError::err)?;

    let public = IconPublic::from(icon);

    info!(uuid = %public.uuid, "update_icon_handler: icon updated");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, payload, pool), fields(icon_uuid = %payload.uuid))]
pub async fn delete_icon_handler(
    token: &str,
    payload: &IconDeleteInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("delete_icon_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    if payload.uuid.trim().is_empty() {
        return Err(ApiError::err("UUID cannot be empty".to_string()));
    }

    icon_soft_delete_query(&payload.uuid, pool)
        .await
        .map_err(ApiError::err)?;

    info!(uuid = %payload.uuid, "delete_icon_handler: icon deleted");
    Ok(ApiResponse::ok(()))
}
