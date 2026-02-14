use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::{Pool, Sqlite};
use tracing::{instrument, error, info};

use crate::api::auth::auth_model::{LoginInput, LoginResponse, AuthConfig};
use crate::api::auth::auth_model::get_auth_config;
use crate::api::auth::auth_tool::verify_password;
use crate::api::user::user_query::user_get_by_email;
use crate::api::user::user_model::UserResponseDB;

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
    inf.insert("uuid".to_string(), user.uuid.clone());
    inf.insert("email".to_string(), user.email.clone());

    let claims = JwtClaims::new(
        Some(config.aud_claims.clone()),
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

