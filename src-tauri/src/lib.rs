pub mod api;
pub mod collector;
pub mod tray;

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
use api::icon::icon_handler::{create_icon_handler, list_icons_handler, get_icon_handler, update_icon_handler, delete_icon_handler};
use api::icon::icon_model::{IconCreateInput, IconListParams, IconListResponse, IconPublic, IconUpdateInput, IconDeleteInput};
use api::device::device_handler::{
    check_device_by_mac_handler, check_device_by_mac_for_adoption_handler, create_device_handler,
    get_device_by_mac_handler, list_devices_handler, delete_device_handler, get_device_handler,
    update_device_handler, DeviceDeleteInput, get_device_commands_for_chart_handler,
};
use api::device::device_model::{
    DeviceCommandChartPoint, DeviceCommandsChartFilter, DeviceCreateInput, DeviceListParams,
    DevicePublic, DeviceUpdateInput,
};
use api::device::provisioning::{
    list_available_ports, probe_device as probe_device_fn, adopt_device as adopt_device_fn,
    get_default_broker_for_adoption as get_default_broker_for_adoption_fn,
    DefaultBrokerInfo, ProbeDeviceInput, AdoptDeviceInput, ProbeDeviceResult, ProvisioningLogEmitter,
    SerialPortInfo,
};
use api::sensor_reading::{
    create_sensor_reading_handler, create_sensor_reading_batch_handler, list_sensor_readings_handler,
    get_sensor_reading_latest_handler, get_sensor_reading_latest_all_handler,
    get_sensor_reading_aggregated_handler, get_sensor_reading_count_handler,
    delete_sensor_reading_old_handler,
    SensorReadingCreateInput, SensorReadingBatchInput, SensorReadingFilter,
    SensorReadingAggregatedFilter, SensorReadingPublic, SensorReadingLatest, SensorReadingAggregated,
};
use api::auth::auth_validator::validate_bearer;
use api::user::user_query::user_get_by_uuid_query;
use collector::persistence::query::{
    count_unread_collector_notifications, get_collector_notification_by_uuid,
    list_collector_notifications_by_user,
    mark_all_collector_notifications_read as mark_all_notifications_read_query,
    mark_collector_notification_read_by_uuid,
    CollectorNotificationListParams, CollectorNotificationListResponse,
};
use collector::service::start_collector;
use collector::state::{CollectorCommand, CollectorState};
use std::sync::Arc;
use tauri::{Emitter, Manager};



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
    collector_state: tauri::State<'_, CollectorState>,
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

            if let Ok(user) = user_get_by_uuid_query(&data.user.uuid, &pool).await {
                if let Err(e) = collector_state
                    .send_command(CollectorCommand::UserLoggedIn { user_id: user.id })
                    .await
                {
                    error!(request_id = %request_id, error = %e, "login: failed to send UserLoggedIn to collector");
                }
            }

            Ok(ApiResponse::ok(data))
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err, "login: error");
            Err(ApiError::err(err))
        },
    }
}

#[tauri::command]
async fn confirm_quit(app_handle: tauri::AppHandle) {
    info!("confirm_quit: frontend cleared session — exiting");
    app_handle.exit(0);
}

#[tauri::command]
async fn logout(collector_state: tauri::State<'_, CollectorState>) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("logout", request_id = %request_id);
    let _guard = span.enter();

    if let Err(e) = collector_state.send_command(CollectorCommand::UserLoggedOut).await {
        error!(request_id = %request_id, error = %e, "logout: failed to send UserLoggedOut to collector");
        return Err(ApiError::err(e));
    }

    info!(request_id = %request_id, "logout: success");
    Ok(ApiResponse::ok(()))
}

#[tauri::command]
async fn connect_broker(
    token: String,
    broker_uuid: String,
    collector_state: tauri::State<'_, CollectorState>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("connect_broker", request_id = %request_id, broker_uuid = %broker_uuid);
    let _guard = span.enter();

    validate_bearer(&token)?;

    if let Err(e) = collector_state
        .send_command(CollectorCommand::ConnectBroker {
            broker_uuid: broker_uuid.clone(),
        })
        .await
    {
        error!(request_id = %request_id, error = %e, "connect_broker: failed to send ConnectBroker");
        return Err(ApiError::err(e));
    }

    info!(request_id = %request_id, broker_uuid = %broker_uuid, "connect_broker: success");
    Ok(ApiResponse::ok(()))
}

#[tauri::command]
async fn disconnect_broker(
    token: String,
    collector_state: tauri::State<'_, CollectorState>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("disconnect_broker", request_id = %request_id);
    let _guard = span.enter();

    validate_bearer(&token)?;

    if let Err(e) = collector_state.send_command(CollectorCommand::DisconnectBroker).await {
        error!(request_id = %request_id, error = %e, "disconnect_broker: failed to send DisconnectBroker");
        return Err(ApiError::err(e));
    }

    info!(request_id = %request_id, "disconnect_broker: success");
    Ok(ApiResponse::ok(()))
}

#[tauri::command]
async fn get_connected_broker_uuid(
    token: String,
    collector_state: tauri::State<'_, CollectorState>,
) -> Result<ApiResponse<Option<String>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("get_connected_broker_uuid", request_id = %request_id);
    let _guard = span.enter();

    validate_bearer(&token)?;

    let uuid = collector_state
        .get_current_broker()
        .map(|b| b.uuid);

    Ok(ApiResponse::ok(uuid))
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

// =====================
// ICON COMMANDS
// =====================

#[tauri::command]
async fn create_icon(
    token: String,
    payload: IconCreateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<IconPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "create_icon",
        request_id = %request_id,
        name = %payload.name,
        iconify_id = %payload.iconify_id,
    );
    let _guard = span.enter();

    match create_icon_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            if let Some(ref icon) = resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %icon.uuid,
                    name = %icon.name,
                    code = %icon.code,
                    "create_icon: success"
                );
            } else {
                info!(request_id = %request_id, "create_icon: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                name = %payload.name,
                iconify_id = %payload.iconify_id,
                "create_icon: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn list_icons(
    token: String,
    params: IconListParams,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<IconListResponse>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "list_icons",
        request_id = %request_id,
        category = ?params.category,
    );
    let _guard = span.enter();

    match list_icons_handler(&token, &params, &pool).await {
        Ok(resp) => {
            info!(
                request_id = %request_id,
                count = resp.data.as_ref().map(|d| d.items.len()).unwrap_or(0),
                total = resp.data.as_ref().map(|d| d.total).unwrap_or(0),
                "list_icons: success"
            );
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                "list_icons: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_icon(
    token: String,
    icon_uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<IconPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_icon",
        request_id = %request_id,
        icon_uuid = %icon_uuid,
    );
    let _guard = span.enter();

    match get_icon_handler(&token, &icon_uuid, &pool).await {
        Ok(resp) => {
            if let Some(ref icon) = resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %icon.uuid,
                    name = %icon.name,
                    "get_icon: success"
                );
            } else {
                info!(request_id = %request_id, "get_icon: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                icon_uuid = %icon_uuid,
                "get_icon: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn update_icon(
    token: String,
    payload: IconUpdateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<IconPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "update_icon",
        request_id = %request_id,
        icon_uuid = %payload.uuid,
    );
    let _guard = span.enter();

    match update_icon_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            if let Some(ref icon) = resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %icon.uuid,
                    name = %icon.name,
                    "update_icon: success"
                );
            } else {
                info!(request_id = %request_id, "update_icon: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                icon_uuid = %payload.uuid,
                "update_icon: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn delete_icon(
    token: String,
    payload: IconDeleteInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "delete_icon",
        request_id = %request_id,
        icon_uuid = %payload.uuid,
    );
    let _guard = span.enter();

    match delete_icon_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            info!(
                request_id = %request_id,
                icon_uuid = %payload.uuid,
                "delete_icon: success"
            );
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                icon_uuid = %payload.uuid,
                "delete_icon: error"
            );
            Err(err)
        }
    }
}

// =====================
// DEVICE COMMANDS
// =====================

#[tauri::command]
async fn create_device(
    token: String,
    payload: DeviceCreateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::device::device_model::DevicePublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "create_device",
        request_id = %request_id,
        name = %payload.name,
        location_uuid = %payload.location_uuid,
    );
    let _guard = span.enter();

    match create_device_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            if let Some(device) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %device.uuid,
                    name = %device.name,
                    "create_device: success"
                );
            } else {
                info!(request_id = %request_id, "create_device: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                name = %payload.name,
                "create_device: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn list_devices(
    token: String,
    params: DeviceListParams,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::device::device_model::DeviceListResponse>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "list_devices",
        request_id = %request_id,
        page = ?params.page,
        page_size = ?params.page_size,
    );
    let _guard = span.enter();

    match list_devices_handler(&token, &params, &pool).await {
        Ok(resp) => {
            info!(
                request_id = %request_id,
                items = resp.data.as_ref().map(|r| r.items.len()).unwrap_or(0),
                total = resp.data.as_ref().map(|r| r.total).unwrap_or(0),
                "list_devices: success"
            );
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                "list_devices: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn delete_device(
    token: String,
    payload: DeviceDeleteInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "delete_device",
        request_id = %request_id,
        device_uuid = %payload.uuid,
    );
    let _guard = span.enter();

    match delete_device_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            info!(request_id = %request_id, "delete_device: success");
            Ok(resp)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "delete_device: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_device(
    token: String,
    device_uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::device::device_model::DevicePublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_device",
        request_id = %request_id,
        device_uuid = %device_uuid,
    );
    let _guard = span.enter();

    match get_device_handler(&token, &device_uuid, &pool).await {
        Ok(resp) => {
            if let Some(device) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %device.uuid,
                    name = %device.name,
                    "get_device: success"
                );
            } else {
                info!(request_id = %request_id, "get_device: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                device_uuid = %device_uuid,
                "get_device: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn check_device_by_mac(
    token: String,
    mac_address: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::device::device_handler::DeviceExistsByMacResponse>, ApiError> {
    check_device_by_mac_handler(&token, &mac_address, &pool).await
}

#[tauri::command]
async fn check_device_by_mac_for_adoption(
    token: String,
    mac_address: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<
    ApiResponse<api::device::device_handler::CheckDeviceByMacForAdoptionResponse>,
    ApiError,
> {
    check_device_by_mac_for_adoption_handler(&token, &mac_address, &pool).await
}

#[tauri::command]
async fn get_device_by_mac(
    token: String,
    mac_address: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<Option<api::device::device_model::DevicePublic>>, ApiError> {
    get_device_by_mac_handler(&token, &mac_address, &pool).await
}

#[tauri::command]
async fn update_device(
    token: String,
    payload: DeviceUpdateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<api::device::device_model::DevicePublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "update_device",
        request_id = %request_id,
        device_uuid = %payload.uuid,
    );
    let _guard = span.enter();

    match update_device_handler(&token, &payload, &pool).await {
        Ok(resp) => {
            if let Some(device) = &resp.data {
                info!(
                    request_id = %request_id,
                    uuid = %device.uuid,
                    name = %device.name,
                    is_active = device.is_active,
                    "update_device: success"
                );
            } else {
                info!(request_id = %request_id, "update_device: success (no data)");
            }
            Ok(resp)
        }
        Err(err) => {
            error!(
                request_id = %request_id,
                error = %err.message,
                device_uuid = %payload.uuid,
                "update_device: error"
            );
            Err(err)
        }
    }
}

#[tauri::command]
async fn list_collector_notifications(
    token: String,
    params: CollectorNotificationListParams,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<CollectorNotificationListResponse>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("list_collector_notifications", request_id = %request_id);
    let _guard = span.enter();

    let auth = validate_bearer(&token)?;
    let user = user_get_by_uuid_query(&auth.user_uuid, pool.inner())
        .await
        .map_err(ApiError::err)?;
    if !user.is_active {
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    info!(request_id = %request_id, page = params.page, page_size = params.page_size, is_read = %params.filter.is_read, severity = %params.filter.severity, "list_collector_notifications: params received");
    match list_collector_notifications_by_user(pool.inner(), user.id, &params).await {
        Ok(resp) => {
            info!(request_id = %request_id, count = resp.items.len(), total = resp.total, "list_collector_notifications: success");
            Ok(ApiResponse::ok(resp))
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "list_collector_notifications: error");
            Err(ApiError::err(e))
        }
    }
}

#[tauri::command]
async fn get_collector_notification(
    token: String,
    uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<
    ApiResponse<Option<collector::persistence::query::CollectorNotificationRow>>,
    ApiError,
> {
    let request_id = Uuid::new_v4();
    let span = info_span!("get_collector_notification", request_id = %request_id, uuid = %uuid);
    let _guard = span.enter();

    let auth = validate_bearer(&token)?;
    let user = user_get_by_uuid_query(&auth.user_uuid, pool.inner())
        .await
        .map_err(ApiError::err)?;
    if !user.is_active {
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    match get_collector_notification_by_uuid(pool.inner(), &uuid, user.id).await {
        Ok(row) => {
            info!(request_id = %request_id, found = row.is_some(), "get_collector_notification: success");
            Ok(ApiResponse::ok(row))
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "get_collector_notification: error");
            Err(ApiError::err(e))
        }
    }
}

#[tauri::command]
async fn mark_collector_notification_read(
    token: String,
    uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("mark_collector_notification_read", request_id = %request_id, uuid = %uuid);
    let _guard = span.enter();

    let auth = validate_bearer(&token)?;
    let user = user_get_by_uuid_query(&auth.user_uuid, pool.inner())
        .await
        .map_err(ApiError::err)?;
    if !user.is_active {
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    match mark_collector_notification_read_by_uuid(pool.inner(), &uuid, user.id).await {
        Ok(()) => {
            info!(request_id = %request_id, "mark_collector_notification_read: success");
            Ok(ApiResponse::ok(()))
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "mark_collector_notification_read: error");
            Err(ApiError::err(e))
        }
    }
}

#[tauri::command]
async fn mark_all_collector_notifications_read(
    token: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<()>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("mark_all_collector_notifications_read", request_id = %request_id);
    let _guard = span.enter();

    let auth = validate_bearer(&token)?;
    let user = user_get_by_uuid_query(&auth.user_uuid, pool.inner())
        .await
        .map_err(ApiError::err)?;
    if !user.is_active {
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    match mark_all_notifications_read_query(pool.inner(), user.id).await {
        Ok(()) => {
            info!(request_id = %request_id, "mark_all_collector_notifications_read: success");
            Ok(ApiResponse::ok(()))
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "mark_all_collector_notifications_read: error");
            Err(ApiError::err(e))
        }
    }
}

#[tauri::command]
async fn count_collector_notifications(
    token: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<i64>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("count_collector_notifications", request_id = %request_id);
    let _guard = span.enter();

    let auth = validate_bearer(&token)?;
    let user = user_get_by_uuid_query(&auth.user_uuid, pool.inner())
        .await
        .map_err(ApiError::err)?;
    if !user.is_active {
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    match count_unread_collector_notifications(pool.inner(), user.id).await {
        Ok(n) => {
            info!(request_id = %request_id, count = n, "count_collector_notifications: success");
            Ok(ApiResponse::ok(n))
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "count_collector_notifications: error");
            Err(ApiError::err(e))
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

// ==================== Device Provisioning ====================

#[tauri::command]
fn list_serial_ports() -> Result<ApiResponse<Vec<SerialPortInfo>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("list_serial_ports", request_id = %request_id);
    let _guard = span.enter();

    match list_available_ports() {
        Ok(ports) => {
            info!(request_id = %request_id, count = ports.len(), "list_serial_ports: success");
            Ok(ApiResponse::ok(ports))
        }
        Err(err) => {
            let msg = err.to_user_message();
            error!(request_id = %request_id, error = %err, "list_serial_ports: error");
            Err(ApiError::err(msg))
        }
    }
}

#[tauri::command]
async fn probe_device(
    app: tauri::AppHandle,
    payload: ProbeDeviceInput,
) -> Result<ApiResponse<ProbeDeviceResult>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "probe_device",
        request_id = %request_id,
        port = %payload.port
    );
    let _guard = span.enter();

    let emit_log: ProvisioningLogEmitter = Arc::new(move |msg| {
        let _ = app.emit("provisioning-log", serde_json::json!({ "message": msg }));
    });

    match probe_device_fn(payload, Some(emit_log)).await {
        Ok(result) => {
            info!(
                request_id = %request_id,
                mac = %result.device_info.mac_address,
                can_adopt = result.can_adopt,
                "probe_device: success"
            );
            Ok(ApiResponse::ok(result))
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err, "probe_device: error");
            Err(ApiError::err(err))
        }
    }
}

#[tauri::command]
async fn adopt_device(
    app: tauri::AppHandle,
    token: String,
    payload: AdoptDeviceInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<DevicePublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "adopt_device",
        request_id = %request_id,
        port = %payload.port,
        name = %payload.name
    );
    let _guard = span.enter();

    let emit_log: ProvisioningLogEmitter = Arc::new({
        let app = app.clone();
        move |msg| {
            let _ = app.emit("provisioning-log", serde_json::json!({ "message": msg }));
        }
    });

    match adopt_device_fn(&token, payload, &pool, Some(emit_log)).await {
        Ok(device) => {
            info!(
                request_id = %request_id,
                uuid = %device.uuid,
                mac = %device.mac_address,
                "adopt_device: success"
            );
            Ok(ApiResponse::ok(device))
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err, "adopt_device: error");
            Err(ApiError::err(err))
        }
    }
}

#[tauri::command]
async fn get_default_broker_for_adoption(
    token: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<Option<DefaultBrokerInfo>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!("get_default_broker_for_adoption", request_id = %request_id);
    let _guard = span.enter();

    match get_default_broker_for_adoption_fn(&token, &pool).await {
        Ok(broker) => {
            info!(request_id = %request_id, "get_default_broker_for_adoption: success");
            Ok(ApiResponse::ok(broker))
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err, "get_default_broker_for_adoption: error");
            Err(ApiError::err(err))
        }
    }
}

// ============================================================================
// Sensor Readings Commands
// ============================================================================

#[tauri::command]
async fn create_sensor_reading(
    token: String,
    payload: SensorReadingCreateInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<SensorReadingPublic>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "create_sensor_reading",
        request_id = %request_id,
        device_uuid = %payload.device_uuid,
        measurement = %payload.measurement
    );
    let _guard = span.enter();

    match create_sensor_reading_handler(&token, &payload, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "create_sensor_reading: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "create_sensor_reading: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn create_sensor_reading_batch(
    token: String,
    payload: SensorReadingBatchInput,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<i64>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "create_sensor_reading_batch",
        request_id = %request_id,
        device_uuid = %payload.device_uuid,
        count = payload.readings.len()
    );
    let _guard = span.enter();

    match create_sensor_reading_batch_handler(&token, &payload, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "create_sensor_reading_batch: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "create_sensor_reading_batch: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn list_sensor_readings(
    token: String,
    filter: SensorReadingFilter,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<Vec<SensorReadingPublic>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "list_sensor_readings",
        request_id = %request_id,
        device_uuid = ?filter.device_uuid
    );
    let _guard = span.enter();

    match list_sensor_readings_handler(&token, &filter, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "list_sensor_readings: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "list_sensor_readings: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_sensor_reading_latest(
    token: String,
    device_uuid: String,
    measurement: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<Option<SensorReadingLatest>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_sensor_reading_latest",
        request_id = %request_id,
        device_uuid = %device_uuid,
        measurement = %measurement
    );
    let _guard = span.enter();

    match get_sensor_reading_latest_handler(&token, &device_uuid, &measurement, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "get_sensor_reading_latest: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "get_sensor_reading_latest: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_sensor_reading_latest_all(
    token: String,
    device_uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<Vec<SensorReadingLatest>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_sensor_reading_latest_all",
        request_id = %request_id,
        device_uuid = %device_uuid
    );
    let _guard = span.enter();

    match get_sensor_reading_latest_all_handler(&token, &device_uuid, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "get_sensor_reading_latest_all: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "get_sensor_reading_latest_all: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_sensor_reading_aggregated(
    token: String,
    filter: SensorReadingAggregatedFilter,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<Vec<SensorReadingAggregated>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_sensor_reading_aggregated",
        request_id = %request_id,
        device_uuid = %filter.device_uuid,
        measurement = %filter.measurement
    );
    let _guard = span.enter();

    match get_sensor_reading_aggregated_handler(&token, &filter, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "get_sensor_reading_aggregated: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "get_sensor_reading_aggregated: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn get_sensor_reading_count(
    token: String,
    device_uuid: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<i64>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_sensor_reading_count",
        request_id = %request_id,
        device_uuid = %device_uuid
    );
    let _guard = span.enter();

    match get_sensor_reading_count_handler(&token, &device_uuid, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "get_sensor_reading_count: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "get_sensor_reading_count: error");
            Err(err)
        }
    }
}

#[tauri::command]
async fn delete_sensor_reading_old(
    token: String,
    device_uuid: String,
    before_date: String,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<u64>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "delete_sensor_reading_old",
        request_id = %request_id,
        device_uuid = %device_uuid,
        before_date = %before_date
    );
    let _guard = span.enter();

    match delete_sensor_reading_old_handler(&token, &device_uuid, &before_date, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "delete_sensor_reading_old: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "delete_sensor_reading_old: error");
            Err(err)
        }
    }
}

// ============================================================================
// Device Commands for Chart (Actuator Dashboard)
// ============================================================================

#[tauri::command]
async fn get_device_commands_for_chart(
    token: String,
    filter: DeviceCommandsChartFilter,
    pool: tauri::State<'_, Pool<Sqlite>>,
) -> Result<ApiResponse<Vec<DeviceCommandChartPoint>>, ApiError> {
    let request_id = Uuid::new_v4();
    let span = info_span!(
        "get_device_commands_for_chart",
        request_id = %request_id,
        device_uuid = %filter.device_uuid
    );
    let _guard = span.enter();

    match get_device_commands_for_chart_handler(&token, &filter, &pool).await {
        Ok(data) => {
            info!(request_id = %request_id, "get_device_commands_for_chart: success");
            Ok(data)
        }
        Err(err) => {
            error!(request_id = %request_id, error = %err.message, "get_device_commands_for_chart: error");
            Err(err)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    let pool = tauri::async_runtime::block_on(get_sqlite_pool());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .manage(pool)
        .invoke_handler(tauri::generate_handler![
            create_user, login, logout, confirm_quit, forgot_password, validate_reset_token, reset_password, change_password,
            connect_broker, disconnect_broker, get_connected_broker_uuid,
            create_location, list_locations, delete_location, update_location, get_location,
            create_mqtt_broker, list_mqtt_brokers, delete_mqtt_broker, get_mqtt_broker, update_mqtt_broker,
            create_icon, list_icons, get_icon, update_icon, delete_icon,
            create_device, list_devices, delete_device, get_device, check_device_by_mac, check_device_by_mac_for_adoption, get_device_by_mac, update_device,
            list_serial_ports, probe_device, adopt_device, get_default_broker_for_adoption,
            create_sensor_reading, create_sensor_reading_batch, list_sensor_readings,
            get_sensor_reading_latest, get_sensor_reading_latest_all, get_sensor_reading_aggregated,
            get_sensor_reading_count, delete_sensor_reading_old,
            get_device_commands_for_chart,
            list_collector_notifications, get_collector_notification, mark_collector_notification_read, mark_all_collector_notifications_read, count_collector_notifications
        ])
        .on_window_event(|app, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Hide window instead of closing (background mode)
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
                api.prevent_close();
            }
        })
        .setup(|app| {
            let handle = app.handle().clone();

            // Create system tray menu
            let menu = tray::create_system_tray_menu(&handle)?;

            // Create and configure tray icon (with custom icon support)
            let tray_builder = tray::create_system_tray_builder(&handle);
            let tray_icon = tray_builder
                .menu(&menu)
                .on_tray_icon_event(tray::handle_tray_icon_event)
                .on_menu_event(tray::handle_menu_event)
                .build(&handle)?;

            // Store tray icon (optional, for future reference)
            let _tray_icon = tray_icon;

            let key_paths = ensure_keys(&handle)?;
            setup_auth_keys(
                key_paths.private_key.to_string_lossy().as_ref(),
                key_paths.public_key.to_string_lossy().as_ref(),
            )
            .map_err(|e| tauri::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            let auth_keys = load_keys_to_memory(&key_paths)?;
            app.manage(auth_keys);

            // Collector: start collector (idle state) + notification listener
            let pool = app.state::<Pool<Sqlite>>().inner().clone();
            let pool_for_listener = pool.clone();
            let collector_result = tauri::async_runtime::block_on(start_collector(pool, handle.clone()))
                .map_err(|e| {
                    tauri::Error::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to start collector: {}", e),
                    ))
                })?;
            tray::notification_handler::start_notification_listener(
                handle.clone(),
                collector_result.notification_rx,
                pool_for_listener,
            );
            app.manage(collector_result.state);

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
