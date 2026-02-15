use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};

use crate::api::error::map_location_db_error;
use crate::api::location::location_model::{Location, LocationCreateDB, LocationListResponse, LocationPublic};

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

#[instrument(skip(pool), fields(user_id = user_id, page = page, page_size = page_size))]
pub async fn location_list_query(
    user_id: i64,
    page: u32,
    page_size: u32,
    pool: &Pool<Sqlite>,
) -> Result<LocationListResponse, String> {
    let limit = page_size as i64;
    let offset = ((page.saturating_sub(1)) * page_size) as i64;

    let items = sqlx::query_as::<_, Location>(
        r#"
        SELECT
            id, uuid, user_id, name, description, address, is_active,
            image_path, thumb_path, image_original_name, image_mime, image_size_bytes, image_checksum_sha256,
            created_at, updated_at
        FROM locations
        WHERE user_id = ?1 AND is_active = 1
        ORDER BY created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
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

    let total = location_count_query(user_id, pool).await?;

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

#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn location_count_query(user_id: i64, pool: &Pool<Sqlite>) -> Result<i64, String> {
    let (total,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) as total FROM locations WHERE user_id = ?1 AND is_active = 1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: count_locations");
        map_location_db_error(&e)
    })?;

    Ok(total)
}

