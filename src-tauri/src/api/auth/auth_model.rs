use std::path::PathBuf;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::api::user::user_model::UserResponseDB;
use crate::api::auth::auth_validator::validate_password_strength;

#[derive(Clone)]
pub struct AuthConfig {
    pub algorithm: String,
    pub aud_claims: String,
    pub exp_claims_additional_sec: usize,
    pub iss_claims: String,
}

#[derive(Clone)]
pub struct PasswordResetConfig {
    pub token_expiration_minutes: i64,
    pub rate_limit_max_attempts: i64,
    pub rate_limit_window_hours: i64,
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
static PASSWORD_RESET_CONFIG: OnceLock<PasswordResetConfig> = OnceLock::new();

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

fn init_password_reset_config() -> PasswordResetConfig {
    PasswordResetConfig {
        token_expiration_minutes: 20,
        rate_limit_max_attempts: 3,
        rate_limit_window_hours: 1,
    }
}

pub fn get_password_reset_config() -> &'static PasswordResetConfig {
    PASSWORD_RESET_CONFIG.get_or_init(init_password_reset_config)
}

// Password Reset Models

/// Internal database model
#[derive(Debug, FromRow)]
pub struct PasswordResetToken {
    pub id: i64,
    pub token: String,
    pub user_id: i64,
    pub expires_at: String,
    pub used_at: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordInput {
    pub email: String,
}

impl ForgotPasswordInput {
    pub fn validate(&self) -> Result<(), String> {
        let email = self.email.trim();
        if email.is_empty() {
            return Err("Email is required".to_string());
        }
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct ResetPasswordInput {
    pub token: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

impl ChangePasswordInput {
    pub fn validate(&self) -> Result<(), String> {
        let current_password = self.current_password.trim();
        if current_password.is_empty() {
            return Err("Current password is required".to_string());
        }

        let new_password = self.new_password.trim();
        if new_password.is_empty() {
            return Err("New password is required".to_string());
        }

        if new_password != self.confirm_password.trim() {
            return Err("Passwords do not match".to_string());
        }

        if new_password == current_password {
            return Err("New password must be different from current password".to_string());
        }

        // Validar força da senha
        validate_password_strength(new_password)?;

        Ok(())
    }
}

impl ResetPasswordInput {
    pub fn validate(&self) -> Result<(), String> {
        let token = self.token.trim();
        if token.is_empty() {
            return Err("Token is required".to_string());
        }

        let password = self.password.trim();
        if password.is_empty() {
            return Err("Password is required".to_string());
        }

        if password != self.confirm_password.trim() {
            return Err("Passwords do not match".to_string());
        }

        // Validar força da senha: mínimo 6 caracteres, letras e pelo menos um número
        if password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }

        let has_letter = password.chars().any(|c| c.is_alphabetic());
        if !has_letter {
            return Err("Password must contain at least one letter".to_string());
        }

        let has_number = password.chars().any(|c| c.is_numeric());
        if !has_number {
            return Err("Password must contain at least one number".to_string());
        }

        Ok(())
    }
}

#[derive(Serialize)]
pub struct ValidateResetTokenPublic {
    pub user_uuid: String,
    pub email: String,
}