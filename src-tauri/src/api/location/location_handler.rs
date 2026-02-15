use sqlx::{Pool, Sqlite};
use tracing::{error, info, instrument};

use tauri::AppHandle;
use crate::api::auth::auth_validator::validate_bearer;
use crate::api::location::location_model::{LocationCreateCommandInput, LocationListParams, LocationListResponse, LocationPublic, LocationDeleteInput};
use crate::api::location::location_query::{location_post_query, location_list_query, location_soft_delete_query};
use crate::api::model::{ApiError, ApiResponse};
use crate::api::user::user_query::user_get_by_uuid_query;
use crate::api::location::location_storage::{save_image_with_thumb, ImagePayload};

#[instrument(skip(token, input, pool, app_handle), fields(name = %input.location.name, address = %input.location.address))]
pub async fn create_location_handler(
    token: &str,
    input: &LocationCreateCommandInput,
    pool: &Pool<Sqlite>,
    app_handle: &AppHandle,
) -> Result<ApiResponse<LocationPublic>, ApiError> {
    // 1) Auth
    let auth = validate_bearer(token)?;

    // 2) Validate payload
    input.location.validate().map_err(ApiError::err)?;

    // 3) Resolve user_id by uuid
    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("create_location_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    // 4) Build DB payload
    let mut db_payload = input.location.to_db(user.id, None, None);

    // 5) Se imagem enviada, salvar arquivo e preencher metadados
    if let Some(img) = &input.image {
        let saved = save_image_with_thumb(
            app_handle,
            &auth.user_uuid,
            &db_payload.uuid,
            ImagePayload {
                data_base64: &img.data_base64,
                original_name: &img.original_name,
                mime: &img.mime,
                size_bytes: img.size_bytes,
            },
        )
        .map_err(ApiError::err)?;
        db_payload.image_path = Some(saved.image_path);
        db_payload.thumb_path = Some(saved.thumb_path);
        db_payload.image_original_name = Some(saved.original_name);
        db_payload.image_mime = Some(saved.mime);
        db_payload.image_size_bytes = Some(saved.size_bytes);
        db_payload.image_checksum_sha256 = Some(saved.checksum_sha256);
    }

    // 6) Insert
    let location = location_post_query(&db_payload, pool)
        .await
        .map_err(ApiError::err)?;

    let public = LocationPublic {
        uuid: location.uuid,
        name: location.name,
        description: location.description,
        address: location.address,
        is_active: location.is_active,
        image_path: location.image_path,
        thumb_path: location.thumb_path,
        image_original_name: location.image_original_name,
        image_mime: location.image_mime,
        image_size_bytes: location.image_size_bytes,
        image_checksum_sha256: location.image_checksum_sha256,
        created_at: location.created_at,
        updated_at: location.updated_at,
    };

    info!(uuid = %public.uuid, "create_location_handler: location created");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, payload, pool), fields(location_uuid = %payload.uuid))]
pub async fn delete_location_handler(
    token: &str,
    payload: &LocationDeleteInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("delete_location_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    location_soft_delete_query(user.id, &payload.uuid, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(()))
}

#[instrument(skip(token, params, pool), fields(page = ?params.page, page_size = ?params.page_size))]
pub async fn list_locations_handler(
    token: &str,
    params: &LocationListParams,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<LocationListResponse>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("list_locations_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 50);

    let resp = location_list_query(user.id, page, page_size, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(resp))
}

