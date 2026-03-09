use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};

use crate::api::error::map_icon_db_error;
use crate::api::icon::icon_model::{Icon, IconCreateDB, IconListParams, IconUpdateDB};
use sqlx::Error as SqlxError;

#[instrument(skip(icon, pool), fields(uuid = %icon.uuid, code = %icon.code))]
pub async fn icon_insert_query(icon: &IconCreateDB, pool: &Pool<Sqlite>) -> Result<Icon, String> {
    let rec = sqlx::query_as::<_, Icon>(
        r#"
        INSERT INTO icons (uuid, code, name, iconify_id, category, color)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        RETURNING id, uuid, code, name, iconify_id, category, color, is_active, created_at, updated_at
        "#,
    )
    .bind(&icon.uuid)
    .bind(&icon.code)
    .bind(&icon.name)
    .bind(&icon.iconify_id)
    .bind(&icon.category)
    .bind(&icon.color)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: icon_insert_query");
        map_icon_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(category = ?params.category, status = ?params.status, page = ?params.page))]
pub async fn icon_list_query(
    params: &IconListParams,
    pool: &Pool<Sqlite>,
) -> Result<(Vec<Icon>, i64), String> {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100);
    let limit = page_size as i64;
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    use crate::api::icon::icon_model::IconStatusFilter;
    let show_all = matches!(params.status, Some(IconStatusFilter::All));

    let (base_where, count_where, category) = match &params.category {
        Some(c) if !c.trim().is_empty() => {
            let cat = c.trim().to_lowercase();
            if cat != "sensor" && cat != "actuator" {
                return Err("category must be sensor or actuator".to_string());
            }
            let active_clause = if show_all { "1=1" } else { "is_active = 1" };
            (
                format!("WHERE {} AND category = ?1", active_clause),
                format!("WHERE {} AND category = ?1", active_clause),
                Some(cat),
            )
        }
        _ => {
            let active_clause = if show_all { "1=1" } else { "is_active = 1" };
            (
                format!("WHERE {}", active_clause),
                format!("WHERE {}", active_clause),
                None,
            )
        }
    };

    let total: i64 = {
        let query_str = format!("SELECT COUNT(*) as total FROM icons {}", count_where.as_str());
        let mut q = sqlx::query_as::<_, (i64,)>(&query_str);
        if let Some(ref cat) = category {
            q = q.bind(cat);
        }
        let (t,) = q
            .fetch_one(pool)
            .await
            .map_err(|e| {
                error!(error = %e, "fn: icon_count");
                map_icon_db_error(&e)
            })?;
        t
    };

    let bind_limit = if category.is_some() { 2 } else { 1 };
    let bind_offset = if category.is_some() { 3 } else { 2 };
    let query_str = format!(
        "SELECT id, uuid, code, name, iconify_id, category, color, is_active, created_at, updated_at
         FROM icons {} ORDER BY id LIMIT ?{} OFFSET ?{}",
        base_where.as_str(), bind_limit, bind_offset
    );
    let mut q = sqlx::query_as::<_, Icon>(&query_str);
    if let Some(ref cat) = category {
        q = q.bind(cat);
    }
    q = q.bind(limit).bind(offset);

    let items = q
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: icon_list_query");
            map_icon_db_error(&e)
        })?;

    Ok((items, total))
}

#[instrument(skip(pool, ids), fields(count = ids.len()))]
pub async fn icon_get_by_ids_query(ids: &[i64], pool: &Pool<Sqlite>) -> Result<Vec<Icon>, String> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders: Vec<String> = (0..ids.len()).map(|i| format!("?{}", i + 1)).collect();
    let query_str = format!(
        "SELECT id, uuid, code, name, iconify_id, category, color, is_active, created_at, updated_at
         FROM icons WHERE id IN ({})",
        placeholders.join(", ")
    );
    let mut q = sqlx::query_as::<_, Icon>(&query_str);
    for id in ids {
        q = q.bind(id);
    }
    let items = q
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: icon_get_by_ids_query");
            map_icon_db_error(&e)
        })?;
    Ok(items)
}

#[instrument(skip(pool), fields(icon_id = %icon_id))]
pub async fn icon_get_by_id_query(icon_id: i64, pool: &Pool<Sqlite>) -> Result<Option<Icon>, String> {
    let rec = sqlx::query_as::<_, Icon>(
        r#"
        SELECT id, uuid, code, name, iconify_id, category, color, is_active, created_at, updated_at
        FROM icons WHERE id = ?1
        "#,
    )
    .bind(icon_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: icon_get_by_id_query");
        map_icon_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(icon_uuid = %icon_uuid))]
pub async fn icon_get_by_uuid_query(
    icon_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<Icon, String> {
    let rec = sqlx::query_as::<_, Icon>(
        r#"
        SELECT id, uuid, code, name, iconify_id, category, color, is_active, created_at, updated_at
        FROM icons WHERE uuid = ?1
        "#,
    )
    .bind(icon_uuid)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "Icon not found".to_string();
        }
        error!(error = %e, "fn: icon_get_by_uuid_query");
        map_icon_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool, update_data), fields(icon_uuid = %icon_uuid))]
pub async fn icon_update_query(
    icon_uuid: &str,
    update_data: &IconUpdateDB,
    pool: &Pool<Sqlite>,
) -> Result<Icon, String> {
    let mut set_clauses = Vec::new();
    let mut bind_index = 1;

    if update_data.name.is_some() {
        set_clauses.push(format!("name = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.iconify_id.is_some() {
        set_clauses.push(format!("iconify_id = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.code.is_some() {
        set_clauses.push(format!("code = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.category.is_some() {
        set_clauses.push(format!("category = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.color.is_some() {
        set_clauses.push(format!("color = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.is_active.is_some() {
        set_clauses.push(format!("is_active = ?{}", bind_index));
        bind_index += 1;
    }

    if set_clauses.is_empty() {
        return Err("No fields to update".to_string());
    }

    let set_clause = set_clauses.join(", ");
    let uuid_bind = bind_index;

    let query_str = format!(
        r#"
        UPDATE icons SET {}
        WHERE uuid = ?{}
        RETURNING id, uuid, code, name, iconify_id, category, color, is_active, created_at, updated_at
        "#,
        set_clause, uuid_bind
    );

    let mut query_builder = sqlx::query_as::<_, Icon>(&query_str);

    if let Some(ref v) = update_data.name {
        query_builder = query_builder.bind(v);
    }
    if let Some(ref v) = update_data.iconify_id {
        query_builder = query_builder.bind(v);
    }
    if let Some(ref v) = update_data.code {
        query_builder = query_builder.bind(v);
    }
    if let Some(ref v) = update_data.category {
        query_builder = query_builder.bind(v);
    }
    if let Some(ref v) = update_data.color {
        query_builder = query_builder.bind(v);
    }
    if let Some(v) = update_data.is_active {
        query_builder = query_builder.bind(v);
    }

    query_builder = query_builder.bind(icon_uuid);

    let rec = query_builder
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let SqlxError::RowNotFound = e {
                return "Icon not found".to_string();
            }
            if let SqlxError::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("2067") {
                    error!(code = ?db_err.code(), message = %db_err, "icon_update_query: unique constraint");
                    return "Icon code already exists".to_string();
                }
            }
            error!(error = %e, "fn: icon_update_query");
            map_icon_db_error(&e)
        })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(icon_uuid = %icon_uuid))]
pub async fn icon_soft_delete_query(
    icon_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    let res = sqlx::query(
        r#"
        UPDATE icons SET is_active = 0 WHERE uuid = ?1 AND is_active = 1
        "#,
    )
    .bind(icon_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: icon_soft_delete_query");
        map_icon_db_error(&e)
    })?;

    if res.rows_affected() == 0 {
        return Err("Icon not found".to_string());
    }

    Ok(())
}
