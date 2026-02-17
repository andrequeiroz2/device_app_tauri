use sqlx::{Pool, Sqlite, Error as SqlxError};
use tracing::{instrument, error};
use crate::api::auth::auth_model::PasswordResetToken;
use crate::api::error::map_db_error;

#[instrument(skip(pool), fields(user_id = user_id, token = %token, expiration_minutes = expiration_minutes))]
pub async fn password_reset_token_create_query(
    token: &str,
    user_id: i64,
    expiration_minutes: i64,
    pool: &Pool<Sqlite>,
) -> Result<PasswordResetToken, String> {
    // Construir a query SQL com o valor de minutos interpolado de forma segura
    // SQLite aceita: datetime('now', '+' || '20' || ' minutes')
    // Mas sqlx pode ter problemas com bind dentro de strings SQL, então vamos construir a query
    let query_str = format!(
        r#"
        INSERT INTO password_reset_tokens (token, user_id, expires_at)
        VALUES (?1, ?2, datetime('now', '+' || '{}' || ' minutes'))
        RETURNING id, token, user_id, expires_at, used_at, created_at
        "#,
        expiration_minutes
    );
    
    let rec = sqlx::query_as::<_, PasswordResetToken>(&query_str)
        .bind(token)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: password_reset_token_create_query");
            map_db_error(&e)
        })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(token = %token))]
pub async fn password_reset_token_get_query(
    token: &str,
    pool: &Pool<Sqlite>,
) -> Result<PasswordResetToken, String> {
    let rec = sqlx::query_as::<_, PasswordResetToken>(
        r#"
        SELECT id, token, user_id, expires_at, used_at, created_at
        FROM password_reset_tokens
        WHERE token = ?1
        LIMIT 1
        "#,
    )
    .bind(token)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let SqlxError::RowNotFound = e {
            return "Invalid or expired token".to_string();
        }
        error!(error = %e, "fn: password_reset_token_get_query");
        map_db_error(&e)
    })?;

    Ok(rec)
}

#[instrument(skip(pool), fields(token = %token))]
pub async fn password_reset_token_mark_used_query(
    token: &str,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    let res = sqlx::query(
        r#"
        UPDATE password_reset_tokens
        SET used_at = CURRENT_TIMESTAMP
        WHERE token = ?1 AND used_at IS NULL
        "#,
    )
    .bind(token)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: password_reset_token_mark_used_query");
        map_db_error(&e)
    })?;

    if res.rows_affected() == 0 {
        return Err("Token not found or already used".to_string());
    }

    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id, window_hours = window_hours))]
pub async fn password_reset_token_count_recent_query(
    user_id: i64,
    window_hours: i64,
    pool: &Pool<Sqlite>,
) -> Result<i64, String> {
    // Contar tokens criados na janela de tempo configurável (para rate limiting)
    let query_str = format!(
        r#"
        SELECT COUNT(*) as count
        FROM password_reset_tokens
        WHERE user_id = ?1 
          AND created_at >= datetime('now', '-' || '{}' || ' hours')
        "#,
        window_hours
    );
    
    let count: i64 = sqlx::query_scalar(&query_str)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "fn: password_reset_token_count_recent_query");
            map_db_error(&e)
        })?;

    Ok(count)
}

#[instrument(skip(pool), fields(token = %token))]
pub async fn password_reset_token_is_expired_query(
    token: &str,
    pool: &Pool<Sqlite>,
) -> Result<bool, String> {
    // Verificar se token expirou comparando expires_at com datetime('now')
    let expired: bool = sqlx::query_scalar(
        r#"
        SELECT datetime(expires_at) < datetime('now') as expired
        FROM password_reset_tokens
        WHERE token = ?1
        "#,
    )
    .bind(token)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "fn: password_reset_token_is_expired_query");
        map_db_error(&e)
    })?;

    Ok(expired)
}

