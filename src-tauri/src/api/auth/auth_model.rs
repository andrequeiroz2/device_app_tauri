use std::path::PathBuf;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use crate::api::user::user_model::UserResponseDB;

#[derive(Clone)]
pub struct AuthConfig {
    pub algorithm: String,
    pub aud_claims: String,
    pub exp_claims_additional_sec: usize,
    pub iss_claims: String,
}

pub struct KeyPairPaths {
    pub private_key: PathBuf,
    pub public_key: PathBuf,
}

#[derive(Clone)]
pub struct AuthKeys {
    pub private_key_pem: String,
    pub public_key_pem: String,
}

#[derive(Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponseDB,
}

static AUTH_CONFIG: OnceLock<AuthConfig> = OnceLock::new();

fn init_auth_config() -> AuthConfig {
    AuthConfig {
        algorithm: "HS256".to_string(),
        aud_claims: "device_app_user".to_string(),
        exp_claims_additional_sec: 3600,
        iss_claims: "device_app_server".to_string(),
    }
}

pub fn get_auth_config() -> &'static AuthConfig {
    AUTH_CONFIG.get_or_init(init_auth_config)
}