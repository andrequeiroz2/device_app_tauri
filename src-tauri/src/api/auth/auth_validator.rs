use serde_json::Value;
use tracing::{error, instrument};

use crate::api::auth::auth_model::get_auth_config;
use crate::api::model::ApiError;
use jwt_lib::jwt_decode;
use jwt_lib::components::claims::JwtClaims;

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_uuid: String,
    pub email: String,
}

#[instrument(skip(token))]
pub fn validate_bearer(token: &str) -> Result<AuthContext, ApiError> {
    let token = extract_token(token)?;
    let config = get_auth_config();

    let claims = jwt_decode(&config.algorithm, token.to_string()).map_err(|e| {
        error!(error = %e, "validate_bearer: jwt_decode failed");
        ApiError::err("Unauthorized".to_string())
    })?;

    let ctx = build_context(&claims)?;
    Ok(ctx)
}

fn extract_token(header: &str) -> Result<&str, ApiError> {
    let trimmed = header.trim();
    let token = if let Some(rest) = trimmed.strip_prefix("Bearer ") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("bearer ") {
        rest
    } else {
        trimmed
    };

    if token.is_empty() {
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    Ok(token)
}

fn build_context(claims: &JwtClaims) -> Result<AuthContext, ApiError> {
    let val = serde_json::to_value(claims).map_err(|e| {
        error!(error = %e, "build_context: serialize claims failed");
        ApiError::err("Unauthorized".to_string())
    })?;

    let user_uuid = val
        .get("sub")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let inf = val.get("inf").and_then(Value::as_object).ok_or_else(|| {
        error!("build_context: missing inf in claims");
        ApiError::err("Unauthorized".to_string())
    })?;

    let email = inf
        .get("email")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    if user_uuid.is_empty() || email.is_empty() {
        error!("build_context: missing subject or email");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    Ok(AuthContext {
        user_uuid,
        email,
    })
}

