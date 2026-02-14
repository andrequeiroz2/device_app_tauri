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
use api::auth::auth_model::{AuthKeys, LoginInput, LoginResponse};
use api::auth::auth_handler::login_handler;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    let pool = tauri::async_runtime::block_on(get_sqlite_pool());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(pool)
        .invoke_handler(tauri::generate_handler![create_user, login])
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
