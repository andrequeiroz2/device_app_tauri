use sqlx::Error;
use scrypt::password_hash::Error as PasswordHashError;
use tracing::error;

pub const INTERNAL_SERVER_ERROR: &str = "Internal server error";

pub fn map_db_error(err: &Error) -> String {
    if let Error::Database(db_err) = err {
        if db_err.code().as_deref() == Some("2067") {
            error!(code = ?db_err.code(), message = %db_err, "db error: unique constraint");
            return "User already registered".to_string();
        }
        error!(code = ?db_err.code(), message = %db_err, "db error");
    } else {
        error!(error = %err, "db error");
    }
    INTERNAL_SERVER_ERROR.to_string()
}

pub fn map_password_hash_error(err: &PasswordHashError) -> String {
    error!(error = %err, "password hash error");
    INTERNAL_SERVER_ERROR.to_string()
}

