
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::{debug, instrument};
use crate::api::user::user_tool::get_password_hash;

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct UserCreateDB {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct UserResponseDB {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow)]
pub struct UserWithPassword {
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

impl CreateUserInput {
    pub fn new(&self) -> Result<UserCreateDB, String> {

        Self::validate_password(&self.password, &self.confirm_password)?;

        let password_hash = get_password_hash(&self.password)?;

        debug!(
            username = %self.username,
            email = %self.email,
            password_hash = %password_hash,
            "struct: CreateUserInput, fn: new"
        );

        Ok(UserCreateDB {
            uuid: uuid::Uuid::new_v4().to_string(),
            username: self.username.to_lowercase().clone(),
            email: self.email.to_lowercase().clone(),
            password: password_hash,
        })
    }

    #[instrument(fields(password = password, confirm_password = confirm_password))]
    fn validate_password(password: &str, confirm_password: &str) -> Result<(), String> {
        
        debug!(
            password = %password,
            confirm_password = %confirm_password,
            "struct: CreateUserInput, fn: validate_password"
        );

        if password != confirm_password {
            debug!(
                password = %password,
                confirm_password = %confirm_password,
                error = "Passwords do not match",
                "struct: CreateUserInput, fn: validate_password"
            );
            return Err("Passwords do not match".to_string());
        }
        Ok(())
    }
}