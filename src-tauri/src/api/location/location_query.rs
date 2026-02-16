use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};

use crate::api::error::map_location_db_error;
use crate::api::location::location_model::{Location, LocationCreateDB, LocationFilter, LocationListResponse, LocationPublic, LocationUpdateDB};
use sqlx::Error as SqlxError;

#[instrument(skip(location, pool), fields(uuid = %location.uuid, user_id = location.user_id, name = %location.name))]
pub async fn location_post_query(
    location: &LocationCreateDB,
    pool: &Pool<Sqlite>,
) -> Result<Location, String> {
    let rec = sqlx::query_as::<_, Location>(
        r#"
        INSERT INTO locations (
            uuid,
            user_id,
            name,
            address,
            description,
            is_active,
            image_path,
            thumb_path,
            image_original_name,
            image_mime,
            image_size_bytes,
            image_checksum_sha256
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        RETURNING
            id,
            uuid,
            user_id,
            name,
            description,
            address,
            is_active,
            image_path,
            thumb_path,
            image_original_name,
            image_mime,
            image_size_bytes,
            image_checksum_sha256,
            created_at,
            updated_at
        "#,
    )
    .bind(&location.uuid)
    .bind(location.user_id)
    .bind(&location.name)
    .bind(&location.address)
    .bind(&location.description)
    .bind(location.is_active)
    .bind(&location.image_path)
    .bind(&location.thumb_path)
    .bind(&location.image_original_name)
    .bind(&location.image_mime)
    .bind(&location.image_size_bytes)
    .bind(&location.image_checksum_sha256)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: location_post_query");
        map_location_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_id = user_id, location_uuid = location_uuid))]
pub async fn location_soft_delete_query(
    user_id: i64,
    location_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    let res = sqlx::query(
        r#"
        UPDATE locations
        SET is_active = 0
        WHERE user_id = ?1 AND uuid = ?2 AND is_active = 1
        "#,
    )
    .bind(user_id)
    .bind(location_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: location_soft_delete_query");
        map_location_db_error(&e)
    })?;

    if res.rows_affected() == 0 {
        return Err("Location not found".to_string());
    }

    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id, page = page, page_size = page_size, filter = ?filter))]
pub async fn location_list_query(
    user_id: i64,
    page: u32,
    page_size: u32,
    filter: &LocationFilter,
    pool: &Pool<Sqlite>,
) -> Result<LocationListResponse, String> {
    let limit = page_size as i64;
    let offset = ((page.saturating_sub(1)) * page_size) as i64;
    
    // Build WHERE condition for status filter
    let is_active_condition = if filter.show_all() {
        "1=1"
    } else {
        "is_active = 1"
    };

    let items = sqlx::query_as::<_, Location>(
        &format!(
            r#"
            SELECT
                id, uuid, user_id, name, description, address, is_active,
                image_path, thumb_path, image_original_name, image_mime, image_size_bytes, image_checksum_sha256,
                created_at, updated_at
            FROM locations
            WHERE user_id = ?1 AND {}
            ORDER BY created_at DESC
            LIMIT ?2 OFFSET ?3
            "#,
            is_active_condition
        ),
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: location_list_query");
        map_location_db_error(&e)
    })?;

    let total = location_count_query(user_id, filter, pool).await?;

    let public_items: Vec<LocationPublic> = items
        .into_iter()
        .map(|loc| LocationPublic {
            uuid: loc.uuid,
            name: loc.name,
            description: loc.description,
            address: loc.address,
            is_active: loc.is_active,
            image_path: loc.image_path,
            thumb_path: loc.thumb_path,
            image_original_name: loc.image_original_name,
            image_mime: loc.image_mime,
            image_size_bytes: loc.image_size_bytes,
            image_checksum_sha256: loc.image_checksum_sha256,
            created_at: loc.created_at,
            updated_at: loc.updated_at,
        })
        .collect();

    Ok(LocationListResponse {
        items: public_items,
        total,
        page,
        page_size,
    })
}

#[instrument(skip(pool), fields(user_id = user_id, filter = ?filter))]
pub async fn location_count_query(
    user_id: i64,
    filter: &LocationFilter,
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    // Build WHERE condition for status filter
    let is_active_condition = if filter.show_all() {
        "1=1"
    } else {
        "is_active = 1"
    };

    let (total,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) as total FROM locations WHERE user_id = ?1 AND {}",
        is_active_condition
    ))
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: location_count_query");
        map_location_db_error(&e)
    })?;

    Ok(total)
}

#[instrument(skip(pool), fields(user_id = user_id, location_uuid = %location_uuid))]
pub async fn location_get_by_uuid_query(
    user_id: i64,
    location_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<Location, String> {
    let rec = sqlx::query_as::<_, Location>(
        r#"
        SELECT
            id, uuid, user_id, name, description, address, is_active,
            image_path, thumb_path, image_original_name, image_mime, image_size_bytes, image_checksum_sha256,
            created_at, updated_at
        FROM locations
        WHERE uuid = ?1 AND user_id = ?2
        "#,
    )
    .bind(location_uuid)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "Location not found".to_string();
        }
        error!(error = %e, "fn: location_get_by_uuid_query");
        map_location_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool, update_data), fields(user_id = user_id, location_uuid = %location_uuid))]
pub async fn location_update_query(
    user_id: i64,
    location_uuid: &str,
    update_data: &LocationUpdateDB,
    pool: &Pool<Sqlite>,
) -> Result<Location, String> {
    // Build dynamic UPDATE query based on provided fields
    let mut set_clauses = Vec::new();
    let mut bind_index = 1;

    if update_data.name.is_some() {
        set_clauses.push(format!("name = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.description.is_some() {
        set_clauses.push(format!("description = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.address.is_some() {
        set_clauses.push(format!("address = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.is_active.is_some() {
        set_clauses.push(format!("is_active = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.image_path.is_some() {
        set_clauses.push(format!("image_path = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.thumb_path.is_some() {
        set_clauses.push(format!("thumb_path = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.image_original_name.is_some() {
        set_clauses.push(format!("image_original_name = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.image_mime.is_some() {
        set_clauses.push(format!("image_mime = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.image_size_bytes.is_some() {
        set_clauses.push(format!("image_size_bytes = ?{}", bind_index));
        bind_index += 1;
    }
    if update_data.image_checksum_sha256.is_some() {
        set_clauses.push(format!("image_checksum_sha256 = ?{}", bind_index));
        bind_index += 1;
    }

    if set_clauses.is_empty() {
        return Err("No fields to update".to_string());
    }

    let set_clause = set_clauses.join(", ");
    let uuid_bind = bind_index;
    let user_id_bind = bind_index + 1;

    let query_str = format!(
        r#"
        UPDATE locations
        SET {}
        WHERE uuid = ?{} AND user_id = ?{}
        RETURNING
            id, uuid, user_id, name, description, address, is_active,
            image_path, thumb_path, image_original_name, image_mime, image_size_bytes, image_checksum_sha256,
            created_at, updated_at
        "#,
        set_clause, uuid_bind, user_id_bind
    );

    let mut query_builder = sqlx::query_as::<_, Location>(&query_str);

    // Bind values in order (same order as set_clauses)
    if let Some(ref name) = update_data.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(ref description) = update_data.description {
        query_builder = query_builder.bind(description);
    }
    if let Some(ref address) = update_data.address {
        query_builder = query_builder.bind(address);
    }
    if let Some(is_active) = update_data.is_active {
        query_builder = query_builder.bind(is_active);
    }
    if let Some(ref image_path) = update_data.image_path {
        query_builder = query_builder.bind(image_path);
    }
    if let Some(ref thumb_path) = update_data.thumb_path {
        query_builder = query_builder.bind(thumb_path);
    }
    if let Some(ref image_original_name) = update_data.image_original_name {
        query_builder = query_builder.bind(image_original_name);
    }
    if let Some(ref image_mime) = update_data.image_mime {
        query_builder = query_builder.bind(image_mime);
    }
    if let Some(image_size_bytes) = update_data.image_size_bytes {
        query_builder = query_builder.bind(image_size_bytes);
    }
    if let Some(ref image_checksum_sha256) = update_data.image_checksum_sha256 {
        query_builder = query_builder.bind(image_checksum_sha256);
    }

    // Bind uuid and user_id
    query_builder = query_builder.bind(location_uuid).bind(user_id);

    let rec = query_builder
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let SqlxError::RowNotFound = e {
                return "Location not found".to_string();
            }
            if let SqlxError::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("2067") {
                    error!(code = ?db_err.code(), message = %db_err, "location_update_query: unique constraint for name");
                    return "Location name already exists".to_string();
                }
            }
            error!(error = %e, "fn: location_update_query");
            map_location_db_error(&e)
        })?;

    Ok(rec)
}

