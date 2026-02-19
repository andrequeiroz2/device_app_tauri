use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::{Pool, Sqlite};
use tracing::{instrument, error, info, warn};
use uuid::Uuid;
use tauri::AppHandle;

use crate::api::auth::auth_model::{LoginInput, LoginResponse, AuthConfig, ForgotPasswordInput, ResetPasswordInput, ValidateResetTokenPublic, ChangePasswordInput};
use crate::api::auth::auth_model::{get_auth_config, get_password_reset_config};
use crate::api::auth::auth_tool::verify_password;
use crate::api::auth::auth_query::{password_reset_token_create_query, password_reset_token_get_query, password_reset_token_mark_used_query, password_reset_token_count_recent_query, password_reset_token_is_expired_query};
use crate::api::auth::auth_validator::{validate_password_strength, validate_bearer};
use crate::api::user::user_query::{user_get_by_email, user_get_by_uuid_query, user_get_by_id_query, user_update_password_query, user_get_by_uuid_with_password_query};
use crate::api::user::user_model::UserResponseDB;
use crate::api::user::get_password_hash;
use crate::api::email::send_reset_password_email;
use crate::api::model::{ApiError, ApiResponse};

use jwt_lib::components::claims::JwtClaims;
use jwt_lib::jwt_encode;

fn now_ts() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

fn exp_ts(config: &AuthConfig) -> usize {

    now_ts() + config.exp_claims_additional_sec
}

#[instrument(skip(payload, pool), fields(email = %payload.email))]
pub async fn login_handler(
    payload: &LoginInput,
    pool: &Pool<Sqlite>,
) -> Result<LoginResponse, String> {

    let user = user_get_by_email(&payload.email, pool).await?;

    verify_password(&payload.password, &user.password)?;

    let config = get_auth_config();
    info!(
        algorithm = %config.algorithm,
        aud_claims = %config.aud_claims,
        iss_claims = %config.iss_claims,
        exp_add_sec = config.exp_claims_additional_sec,
        "login_handler: auth config loaded"
    );
    let now = now_ts();
    let exp = exp_ts(config);

    let mut inf = HashMap::new();
    inf.insert("user_uuid".to_string(), user.uuid.clone());
    inf.insert("email".to_string(), user.email.clone());

    let claims = JwtClaims::new(
        None, // aud: não validar audience para evitar InvalidAudience
        exp,
        Some(now),
        Some(config.iss_claims.clone()),
        None,
        Some(user.uuid.clone()),
        Some(inf),
    ).map_err(|e| {
        error!(error = %e, "login_handler: claims build failed");
        "Internal server error".to_string()
    })?;

    let token = jwt_encode(&config.algorithm, claims).map_err(|e| {
        error!(error = %e, "login_handler: jwt_encode failed");
        "Internal server error".to_string()
    })?;

    let user_response = UserResponseDB {
        uuid: user.uuid,
        username: user.username,
        email: user.email,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
    };

    Ok(LoginResponse { token, user: user_response })
}

#[instrument(skip(input, pool, app_handle), fields(email = %input.email))]
pub async fn forgot_password_handler(
    input: &ForgotPasswordInput,
    pool: &Pool<Sqlite>,
    app_handle: &AppHandle,
) -> Result<ApiResponse<()>, ApiError> {
    // 1) Validar input
    input.validate().map_err(ApiError::err)?;

    // 2) Buscar usuário por email (não expor se não existe)
    let email_lower = input.email.trim().to_lowercase();
    let user = match user_get_by_email(&email_lower, pool).await {
        Ok(u) => u,
        Err(_) => {
            // Por segurança, sempre retornar sucesso mesmo se email não existir
            info!(email = %email_lower, "forgot_password_handler: email not found (returning success for security)");
            return Ok(ApiResponse::ok(()));
        }
    };

    // 3) Verificar se usuário está ativo
    if !user.is_active {
        warn!(email = %email_lower, "forgot_password_handler: user inactive (returning success for security)");
        return Ok(ApiResponse::ok(()));
    }

    // 4) Buscar User completo para ter o id
    let user_full = user_get_by_uuid_query(&user.uuid, pool)
        .await
        .map_err(|e| {
            error!(error = %e, user_uuid = %user.uuid, "forgot_password_handler: user not found");
            ApiError::err("Internal server error".to_string())
        })?;

    // 5) Obter configuração de password reset
    let reset_config = get_password_reset_config();

    // 6) Verificar rate limiting (configurável)
    let recent_count = password_reset_token_count_recent_query(
        user_full.id,
        reset_config.rate_limit_window_hours,
        pool,
    )
    .await
    .map_err(ApiError::err)?;

    if recent_count >= reset_config.rate_limit_max_attempts {
        error!(
            email = %email_lower,
            count = recent_count,
            max_attempts = reset_config.rate_limit_max_attempts,
            window_hours = reset_config.rate_limit_window_hours,
            "forgot_password_handler: rate limit exceeded"
        );
        return Err(ApiError::err(format!(
            "Too many password reset attempts. Please wait {} hour(s) before trying again.",
            reset_config.rate_limit_window_hours
        )));
    }

    // 7) Gerar token único (UUID v4)
    let token = Uuid::new_v4().to_string();

    // 8) Salvar token no banco (expiração configurável)
    password_reset_token_create_query(&token, user_full.id, reset_config.token_expiration_minutes, pool)
        .await
        .map_err(ApiError::err)?;

    info!(email = %email_lower, token = %token, "forgot_password_handler: token created");

    // 9) Enviar email via Resend
    match send_reset_password_email(app_handle, &user.email, &token).await {
        Ok(_) => {
            info!(email = %email_lower, "forgot_password_handler: email sent successfully");
        }
        Err(e) => {
            // Logar erro mas não falhar a requisição (por segurança, sempre retornar sucesso)
            error!(error = %e, email = %email_lower, "forgot_password_handler: failed to send email");
            // Continuar e retornar sucesso mesmo se email falhar (por segurança)
        }
    }

    // Por segurança, sempre retornar sucesso
    Ok(ApiResponse::ok(()))
}

#[instrument(skip(pool), fields(token = %token))]
pub async fn validate_reset_token_handler(
    token: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<ValidateResetTokenPublic>, ApiError> {
    // 1) Buscar token no banco
    let reset_token = password_reset_token_get_query(token, pool)
        .await
        .map_err(|e| {
            error!(error = %e, "validate_reset_token_handler: token not found");
            ApiError::err("Invalid or expired token".to_string())
        })?;

    // 2) Verificar se token já foi usado
    if reset_token.used_at.is_some() {
        error!(token = %token, "validate_reset_token_handler: token already used");
        return Err(ApiError::err("Token has already been used".to_string()));
    }

    // 3) Verificar se token expirou
    let expired = password_reset_token_is_expired_query(token, pool)
        .await
        .map_err(|e| {
            error!(error = %e, "validate_reset_token_handler: expiration check failed");
            ApiError::err("Internal server error".to_string())
        })?;

    if expired {
        error!(token = %token, expires_at = %reset_token.expires_at, "validate_reset_token_handler: token expired");
        return Err(ApiError::err("Token has expired".to_string()));
    }

    // 4) Buscar usuário pelo user_id do token
    let user = user_get_by_id_query(reset_token.user_id, pool)
        .await
        .map_err(|e| {
            error!(error = %e, user_id = reset_token.user_id, "validate_reset_token_handler: user not found");
            ApiError::err("User not found".to_string())
        })?;

    let response = ValidateResetTokenPublic {
        user_uuid: user.uuid,
        email: user.email,
    };

    info!(user_uuid = %response.user_uuid, email = %response.email, "validate_reset_token_handler: token validated");
    Ok(ApiResponse::ok(response))
}

#[instrument(skip(input, pool), fields(token = %input.token))]
pub async fn reset_password_handler(
    input: &ResetPasswordInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    // 1) Validar input (inclui validação de senha)
    input.validate().map_err(ApiError::err)?;

    // 2) Buscar e validar token
    let reset_token = password_reset_token_get_query(&input.token, pool)
        .await
        .map_err(|e| {
            error!(error = %e, "reset_password_handler: token not found");
            ApiError::err("Invalid or expired token".to_string())
        })?;

    // 3) Verificar se token já foi usado
    if reset_token.used_at.is_some() {
        error!(token = %input.token, "reset_password_handler: token already used");
        return Err(ApiError::err("Token has already been used".to_string()));
    }

    // 4) Verificar se token expirou
    let expired = password_reset_token_is_expired_query(&input.token, pool)
        .await
        .map_err(|e| {
            error!(error = %e, "reset_password_handler: expiration check failed");
            ApiError::err("Internal server error".to_string())
        })?;

    if expired {
        error!(token = %input.token, expires_at = %reset_token.expires_at, "reset_password_handler: token expired");
        return Err(ApiError::err("Token has expired".to_string()));
    }

    // 5) Buscar usuário pelo user_id do token
    let user = user_get_by_id_query(reset_token.user_id, pool)
        .await
        .map_err(|e| {
            error!(error = %e, user_id = reset_token.user_id, "reset_password_handler: user not found");
            ApiError::err("User not found".to_string())
        })?;

    // 6) Validar força da senha (usando função reutilizável)
    validate_password_strength(&input.password)
        .map_err(ApiError::err)?;

    // 7) Hash da nova senha
    let password_hash = get_password_hash(&input.password)
        .map_err(|e| {
            error!(error = %e, "reset_password_handler: password hash failed");
            ApiError::err("Internal server error".to_string())
        })?;

    // 8) Atualizar senha do usuário
    user_update_password_query(&user.uuid, &password_hash, pool)
        .await
        .map_err(ApiError::err)?;

    // 9) Marcar token como usado
    password_reset_token_mark_used_query(&input.token, pool)
        .await
        .map_err(ApiError::err)?;

    info!(user_uuid = %user.uuid, email = %user.email, "reset_password_handler: password reset successful");
    Ok(ApiResponse::ok(()))
}

#[instrument(skip(token, input, pool))]
pub async fn change_password_handler(
    token: &str,
    input: &ChangePasswordInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    // 1) Autenticar usuário
    let auth = validate_bearer(token)?;

    // 2) Validar input (inclui validação de senha e verificação de diferença)
    input.validate().map_err(ApiError::err)?;

    // 3) Buscar usuário com senha pelo UUID
    let user = user_get_by_uuid_with_password_query(&auth.user_uuid, pool)
        .await
        .map_err(|e| {
            error!(error = %e, user_uuid = %auth.user_uuid, "change_password_handler: user not found");
            ApiError::err("User not found".to_string())
        })?;

    // 4) Verificar se usuário está ativo
    if !user.is_active {
        error!(user_uuid = %auth.user_uuid, "change_password_handler: user inactive");
        return Err(ApiError::err("User account is inactive".to_string()));
    }

    // 5) Verificar senha atual
    verify_password(&input.current_password, &user.password)
        .map_err(|e| {
            error!(user_uuid = %auth.user_uuid, "change_password_handler: current password incorrect");
            ApiError::err(e)
        })?;

    // 6) Hash da nova senha
    let password_hash = get_password_hash(&input.new_password)
        .map_err(|e| {
            error!(error = %e, "change_password_handler: password hash failed");
            ApiError::err("Internal server error".to_string())
        })?;

    // 7) Atualizar senha do usuário
    user_update_password_query(&user.uuid, &password_hash, pool)
        .await
        .map_err(ApiError::err)?;

    info!(user_uuid = %user.uuid, email = %user.email, "change_password_handler: password changed successfully");
    Ok(ApiResponse::ok(()))
}

