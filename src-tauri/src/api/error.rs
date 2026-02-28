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

pub fn map_location_db_error(err: &Error) -> String {
    if let Error::Database(db_err) = err {
        if db_err.code().as_deref() == Some("2067") {
            error!(code = ?db_err.code(), message = %db_err, "location db error: unique constraint");
            return "Location already exists".to_string();
        }
        error!(code = ?db_err.code(), message = %db_err, "location db error");
    } else {
        error!(error = %err, "location db error");
    }
    INTERNAL_SERVER_ERROR.to_string()
}

pub fn map_mqtt_broker_db_error(err: &Error) -> String {
    if let Error::Database(db_err) = err {
        if db_err.code().as_deref() == Some("2067") {
            error!(code = ?db_err.code(), message = %db_err, "mqtt_broker db error: unique constraint");

            let error_msg = db_err.to_string().to_lowercase();
            if error_msg.contains("name") {
                return "Broker name already exists".to_string();
            }
            if error_msg.contains("host") || error_msg.contains("port") {
                return "Broker with this host and port already exists".to_string();
            }
            return "Broker already exists".to_string();
        }
        error!(code = ?db_err.code(), message = %db_err, "mqtt_broker db error");
    } else {
        error!(error = %err, "mqtt_broker db error");
    }
    INTERNAL_SERVER_ERROR.to_string()
}

pub fn map_device_db_error(err: &Error) -> String {
    if let Error::Database(db_err) = err {
        if db_err.code().as_deref() == Some("2067") {
            error!(code = ?db_err.code(), message = %db_err, "device db error: unique constraint");

            let error_msg = db_err.to_string().to_lowercase();
            if error_msg.contains("name") {
                return "Device name already exists".to_string();
            }
            if error_msg.contains("mac_address") {
                return "Device with this MAC address already exists".to_string();
            }
            return "Device already exists".to_string();
        }
        error!(code = ?db_err.code(), message = %db_err, "device db error");
    } else {
        error!(error = %err, "device db error");
    }
    INTERNAL_SERVER_ERROR.to_string()
}

