use sqlx::{Pool, Sqlite};
use crate::api::user::user_model::{UserCreateDB, UserResponseDB, UserWithPassword};
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