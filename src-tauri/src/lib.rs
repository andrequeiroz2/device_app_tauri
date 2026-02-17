pub mod api;

use sqlx::{Pool, Sqlite};
use api::database::connect_sqlite::get_sqlite_pool;
use api::user::user_handler::user_create_handler;
use api::model::{ApiResponse, ApiError};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;
use tracing_error::ErrorLayer;
use tracing::{info_span, info, error};
use uuid::Uuid;
use api::auth::auth_tool::{ensure_keys, setup_auth_keys, load_keys_to_memory};
use api::auth::auth_model::{AuthKeys, LoginInput, LoginResponse, ForgotPasswordInput, ResetPasswordInput, ValidateResetTokenPublic, ChangePasswordInput};
use api::auth::auth_handler::{login_handler, forgot_password_handler, validate_reset_token_handler, reset_password_handler, change_password_handler};
use api::location::location_handler::create_location_handler;
use api::location::location_handler::list_locations_handler;
use api::location::location_handler::delete_location_handler;
use api::location::location_handler::update_location_handler;
use api::location::location_handler::get_location_handler;
use api::location::location_model::{LocationCreateCommandInput, LocationListParams};
use api::location::location_model::LocationDeleteInput;
use api::location::location_model::LocationUpdateInput;
use api::mqtt_broker::mqtt_broker_handler::{create_mqtt_broker_handler, list_mqtt_brokers_handler, delete_mqtt_broker_handler, get_mqtt_broker_handler, update_mqtt_broker_handler};
use api::mqtt_broker::mqtt_broker_model::{MqttBrokerCreateInput, MqttBrokerListParams, MqttBrokerDeleteInput, MqttBrokerUpdateInput};
use tauri::Manager;



#[tauri::command]
async fn create_user(
    payload: api::user::user_model::CreateUserInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::user::user_model::UserResponseDB>, ApiError> {
    
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "create_user",
        request_id = %request_id,
        username = %payload.username,
        email = %payload.email
    );
    let _guard = span.enter();

    match user_create_handler(&payload, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, uuid = %data.uuid, email = %data.email, "create_user: success");
            Ok(ApiResponse::ok(data))
        },
        Err(err) => {
            error!(request_id = %request_id, error = %err, "create_user: error");
            Err(ApiError::err(err))
        },
    }
}

#[tauri::command]
async fn login(
    payload: LoginInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
    _auth_keys: tauri::State<'_, AuthKeys>,
) -> Result<ApiResponse<LoginResponse>, ApiError> {

    let request_id = Uuid::new_v4();
    let span = info_span!(
        "login",
        request_id = %request_id,
        email = %payload.email
    );
    let _guard = span.enter();

    match login_handler(&payload, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, uuid = %data.user.uuid, email = %data.user.email, token = %data.token, "login: success");
            Ok(ApiResponse::ok(data))
        },
        Err(err) => {
            error!(request_id = %request_id, error = %err, "login: error");
            Err(ApiError::err(err))
        },
    }
}

#[tauri::command]
async fn create_location(
    token: String,
    payload: LocationCreateCommandInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<api::location::location_model::LocationPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "create_location",
        request_id = %request_id,
        name = %payload.location.name,
        address = %payload.location.address,
    );
    let _guard = span.enter();

    match create_location_handler(&token, &payload, &pool, &app_handle).await {
        Ok(resp) => {
            if let Some(loc) = &resp.data {
                info!(request_id = %request_id, uuid = %loc.uuid, "create_location: success");
            } else {
                info!(request_id = %request_id, "create_location: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "create_location: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn list_locations(
    token: String,
    params: LocationListParams,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::location::location_model::LocationListResponse>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "list_locations",
        request_id = %request_id,
        page = ?params.page,
        page_size = ?params.page_size,
    );
    let _guard = span.enter();

    match list_locations_handler(&token, &params, &pool).await {
        Ok(resp) => {
            info!(request_id = %request_id, items = resp.data.as_ref().map(|r| r.items.len()).unwrap_or(0), total = resp.data.as_ref().map(|r| r.total).unwrap_or(0), "list_locations: success");
            Ok(resp)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "list_locations: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn delete_location(
    token: String,
    payload: LocationDeleteInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "delete_location",
        request_id = %request_id,
        location_uuid = %payload.uuid,
    );
    let _guard = span.enter();

    match delete_location_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            info!(request_id = %request_id, "delete_location: success");
            Ok(resp)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "delete_location: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn update_location(
    token: String,
    payload: LocationUpdateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<api::location::location_model::LocationPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "update_location",
        request_id = %request_id,
        location_uuid = %payload.uuid,
    );
    let _guard = span.enter();

    match update_location_handler(&token, &payload, &pool, &app_handle).await {
        Ok(resp) => {
            if let Some(loc) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %loc.uuid,
                    name = %loc.name,
                    is_active = loc.is_active,
                    "update_location: success"
                );
            } else {
                info!(request_id = %request_id, "update_location: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                location_uuid = %payload.uuid,
                "update_location: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_location(
    token: String,
    location_uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::location::location_model::LocationPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_location",
        request_id = %request_id,
        location_uuid = %location_uuid,
    );
    let _guard = span.enter();

    match get_location_handler(&token, &location_uuid, &pool).await {
        Ok(resp) => {
            if let Some(loc) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %loc.uuid,
                    name = %loc.name,
                    "get_location: success"
                );
            } else {
                info!(request_id = %request_id, "get_location: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                location_uuid = %location_uuid,
                "get_location: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn create_mqtt_broker(
    token: String,
    payload: MqttBrokerCreateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::mqtt_broker::mqtt_broker_model::MqttBrokerPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "create_mqtt_broker",
        request_id = %request_id,
        name = %payload.name,
        host = %payload.host,
        port = ?payload.port,
    );
    let _guard = span.enter();

    match create_mqtt_broker_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            if let Some(broker) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %broker.uuid,
                    name = %broker.name,
                    host = %broker.host,
                    port = broker.port,
                    is_default = broker.is_default,
                    "create_mqtt_broker: success"
                );
            } else {
                info!(request_id = %request_id, "create_mqtt_broker: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                name = %payload.name,
                host = %payload.host,
                "create_mqtt_broker: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn list_mqtt_brokers(
    token: String,
    params: MqttBrokerListParams,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::mqtt_broker::mqtt_broker_model::MqttBrokerListResponse>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "list_mqtt_brokers",
        request_id = %request_id,
        page = ?params.page,
        page_size = ?params.page_size,
    );
    let _guard = span.enter();

    match list_mqtt_brokers_handler(&token, &params, &pool).await {
        Ok(resp) => {
            info!(
                request_id = %request_id,
                items = resp.data.as_ref().map(|r| r.items.len()).unwrap_or(0),
                total = resp.data.as_ref().map(|r| r.total).unwrap_or(0),
                "list_mqtt_brokers: success"
            );
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                "list_mqtt_brokers: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn delete_mqtt_broker(
    token: String,
    payload: MqttBrokerDeleteInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "delete_mqtt_broker",
        request_id = %request_id,
        broker_uuid = %payload.uuid,
    );
    let _guard = span.enter();

    match delete_mqtt_broker_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            info!(request_id = %request_id, "delete_mqtt_broker: success");
            Ok(resp)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "delete_mqtt_broker: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_mqtt_broker(
    token: String,
    broker_uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::mqtt_broker::mqtt_broker_model::MqttBrokerPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_mqtt_broker",
        request_id = %request_id,
        broker_uuid = %broker_uuid,
    );
    let _guard = span.enter();

    match get_mqtt_broker_handler(&token, &broker_uuid, &pool).await {
        Ok(resp) => {
            if let Some(broker) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %broker.uuid,
                    name = %broker.name,
                    "get_mqtt_broker: success"
                );
            } else {
                info!(request_id = %request_id, "get_mqtt_broker: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                broker_uuid = %broker_uuid,
                "get_mqtt_broker: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn update_mqtt_broker(
    token: String,
    payload: MqttBrokerUpdateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::mqtt_broker::mqtt_broker_model::MqttBrokerPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "update_mqtt_broker",
        request_id = %request_id,
        broker_uuid = %payload.uuid,
        is_active = ?payload.is_active,
    );
    let _guard = span.enter();

    match update_mqtt_broker_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            if let Some(broker) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %broker.uuid,
                    name = %broker.name,
                    is_active = broker.is_active,
                    "update_mqtt_broker: success"
                );
            } else {
                info!(request_id = %request_id, "update_mqtt_broker: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                broker_uuid = %payload.uuid,
                "update_mqtt_broker: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn forgot_password(
    payload: ForgotPasswordInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "forgot_password",
        request_id = %request_id,
        email = %payload.email
    );
    let _guard = span.enter();

    match forgot_password_handler(&payload, &pool, &app_handle).await {
        Ok(data) => {
            info!(request_id = %request_id, email = %payload.email, "forgot_password: success");
            Ok(data)
        },
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "forgot_password: error");
            Err(err)
        },
    }
}

#[tauri::command]
async fn validate_reset_token(
    token: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<ValidateResetTokenPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "validate_reset_token",
        request_id = %request_id,
        token = %token
    );
    let _guard = span.enter();

    match validate_reset_token_handler(&token, &pool).await {
        Ok(data) => {
            if let Some(token_data) = &data.data {
                info!(
                    request_id = %request_id,
                    user_uuid = %token_data.user_uuid,
                    email = %token_data.email,
                    "validate_reset_token: success"
                );
            } else {
                info!(request_id = %request_id, "validate_reset_token: success (no data)");
            }
            Ok(data)
        },
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "validate_reset_token: error");
            Err(err)
        },
    }
}

#[tauri::command]
async fn reset_password(
    payload: ResetPasswordInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "reset_password",
        request_id = %request_id,
        token = %payload.token
    );
    let _guard = span.enter();

    match reset_password_handler(&payload, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "reset_password: success");
            Ok(data)
        },
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "reset_password: error");
            Err(err)
        },
    }
}

#[tauri::command]
async fn change_password(
    token: String,
    payload: ChangePasswordInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "change_password",
        request_id = %request_id
    );
    let _guard = span.enter();

    match change_password_handler(&token, &payload, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "change_password: success");
            Ok(data)
        },
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "change_password: error");
            Err(err)
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    let pool = tauri::async_runtime::block_on(get_sqlite_pool());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pool)
        .invoke_handler(tauri::generate_handler![create_user, login, forgot_password, validate_reset_token, reset_password, change_password, create_location, list_locations, delete_location, update_location, get_location, create_mqtt_broker, list_mqtt_brokers, delete_mqtt_broker, get_mqtt_broker, update_mqtt_broker])
        .setup(|app| {
            let handle = app.handle();
            let key_paths = ensure_keys(&handle)?;
            setup_auth_keys(
                key_paths.private_key.to_string_lossy().as_ref(),
                key_paths.public_key.to_string_lossy().as_ref(),
            ).map_err(|e| tauri::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            let auth_keys = load_keys_to_memory(&key_paths)?;
            app.manage(auth_keys);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_tracing() {
    // Use RUST_LOG para ajustar nível, ex.: RUST_LOG=info,device_app=debug
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(env_filter)
        .with(ErrorLayer::default())
        .with(fmt::layer().with_target(true).with_line_number(true))
        .init();
}
