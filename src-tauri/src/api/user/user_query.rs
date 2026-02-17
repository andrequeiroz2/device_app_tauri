use sqlx::{Pool, Sqlite};
use crate::api::user::user_model::{UserCreateDB, UserResponseDB, UserWithPassword, User};
use crate::api::error::map_db_error;
use sqlx::Error as SqlxError;
use tracing::{instrument, error};

#[instrument(skip(user, pool), fields(uuid = %user.uuid, username = %user.username, email = %user.email, password = %user.password))]
pub async fn user_post_query(
    user: &UserCreateDB,
    pool: &Pool<Sqlite>,
) -> Result<UserResponseDB, String> {

    let rec = sqlx::query_as::<_, UserResponseDB>(
        "INSERT INTO users (uuid, username, email, password)
         VALUES (?1, ?2, ?3, ?4)
         RETURNING uuid, username, email, is_active, created_at, updated_at",
    )
        .bind(&user.uuid)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: user_post_query");
            map_db_error(&e)
        })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_uuid = %uuid))]
pub async fn user_get_by_uuid_query(
    uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<User, String> {
    let rec = sqlx::query_as::<_, User>(
        "SELECT id, uuid, username, email, is_active, created_at, updated_at FROM users WHERE uuid = ?1 LIMIT 1",
    )
    .bind(uuid)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "User not found".to_string();
        }
        error!(error = %e, "fn: user_get_by_uuid_query");
        map_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(email = %email))]
pub async fn user_get_by_email(
    email: &str,
    pool: &Pool<Sqlite>,
) -> Result<UserWithPassword, String> {

    let rec = sqlx::query_as::<_, UserWithPassword>(
        "SELECT uuid, username, email, password, is_active, created_at, updated_at
         FROM users
         WHERE email = ?1
         LIMIT 1",
    )
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let SqlxError::RowNotFound = e {
                return "Incorrect email or password".to_string();
            }
            error!(error = %e, "fn: user_get_by_email");
            map_db_error(&e)
        })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn user_get_by_id_query(
    user_id: i64,
    pool: &Pool<Sqlite>,
) -> Result<User, String> {
    let rec = sqlx::query_as::<_, User>(
        r#"
        SELECT id, uuid, username, email, is_active, created_at, updated_at
        FROM users
        WHERE id = ?1
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "User not found".to_string();
        }
        error!(error = %e, "fn: user_get_by_id_query");
        map_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(user_uuid = %user_uuid))]
pub async fn user_update_password_query(
    user_uuid: &str,
    password_hash: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    let res = sqlx::query(
        r#"
        UPDATE users
        SET password = ?1
        WHERE uuid = ?2
        "#,
    )
    .bind(password_hash)
    .bind(user_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: user_update_password_query");
        map_db_error(&e)
    })?;

    if res.rows_affected() == 0 {
        return Err("User not found".to_string());
    }

    Ok(())
}

#[instrument(skip(pool), fields(user_uuid = %user_uuid))]
pub async fn user_get_by_uuid_with_password_query(
    user_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<UserWithPassword, String> {
    let rec = sqlx::query_as::<_, UserWithPassword>(
        r#"
        SELECT uuid, username, email, password, is_active, created_at, updated_at
        FROM users
        WHERE uuid = ?1
        LIMIT 1
        "#,
    )
    .bind(user_uuid)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "User not found".to_string();
        }
        error!(error = %e, "fn: user_get_by_uuid_with_password_query");
        map_db_error(&e)
    })?;

    Ok(rec)
}