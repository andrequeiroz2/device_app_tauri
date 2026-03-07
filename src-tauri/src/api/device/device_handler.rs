use sqlx::{Pool, Sqlite};
use tracing::{debug, error, info, instrument};

use crate::api::auth::auth_validator::validate_bearer;
use crate::api::device::device_model::{
    parse_device_type, parse_operation_status, Device, DeviceCreateInput,
    DeviceListParams, DeviceListResponse, DevicePublic, DeviceUpdateInput,
};
use crate::api::device::device_query::{
    device_count_query, device_get_by_mac_any_user_query, device_get_by_mac_query,
    device_get_by_uuid_query, device_list_query, device_post_query, device_soft_delete_query,
    device_update_query, get_location_uuid_by_id, get_user_uuid_by_id,
};
use crate::api::location::location_query::location_get_by_uuid_query;
use crate::api::model::{ApiError, ApiResponse};
use crate::api::user::user_query::user_get_by_uuid_query;
use crate::api::device::device_model::{DeviceCommandChartPoint, DeviceCommandsChartFilter};
use crate::api::device::device_query::device_commands_for_chart_query;

fn device_to_public(device: Device, user_uuid: String, location_uuid: String) -> DevicePublic {
    let device_scale = device
        .device_scale
        .as_ref()
        .and_then(|s| serde_json::from_str(s).ok());

    DevicePublic {
        uuid: device.uuid,
        user_uuid,
        location_uuid,
        name: device.name,
        description: device.description,
        device_type: parse_device_type(&device.device_type),
        model: device.model,
        firmware_version: device.firmware_version,
        mac_address: device.mac_address,
        sensor_type: device.sensor_type,
        actuator_type: device.actuator_type,
        device_scale,
        adopted_at: device.adopted_at,
        operation_status: parse_operation_status(device.operation_status.as_deref()),
        last_seen_at: device.last_seen_at,
        ip_address: device.ip_address,
        publish_qos: device.publish_qos,
        subscribe_qos: device.subscribe_qos,
        status_retain: device.status_retain,
        data_retain: device.data_retain,
        lwt_enabled: device.lwt_enabled,
        lwt_message: device.lwt_message,
        lwt_qos: device.lwt_qos,
        lwt_retain: device.lwt_retain,
        heartbeat_interval: device.heartbeat_interval,
        offline_threshold: device.offline_threshold,
        last_command: device.last_command,
        last_command_at: device.last_command_at,
        is_active: device.is_active,
        created_at: device.created_at,
        updated_at: device.updated_at,
    }
}

#[instrument(skip(token, input, pool), fields(name = %input.name))]
pub async fn create_device_handler(
    token: &str,
    input: &DeviceCreateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<DevicePublic>, ApiError> {
    let auth = validate_bearer(token)?;

    input.validate().map_err(ApiError::err)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("create_device_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let location = location_get_by_uuid_query(user.id, &input.location_uuid, pool)
        .await
        .map_err(ApiError::err)?;
    let location_id = location.id;

    let db_payload = input.to_db(user.id, location_id);

    let device = device_post_query(&db_payload, pool)
        .await
        .map_err(ApiError::err)?;

    let public = device_to_public(device, auth.user_uuid.clone(), input.location_uuid.clone());

    info!(uuid = %public.uuid, "create_device_handler: device created");
    Ok(ApiResponse::ok(public))
}

#[derive(Debug, serde::Deserialize)]
pub struct DeviceDeleteInput {
    pub uuid: String,
}

#[instrument(skip(token, payload, pool), fields(device_uuid = %payload.uuid))]
pub async fn delete_device_handler(
    token: &str,
    payload: &DeviceDeleteInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<()>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("delete_device_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    device_soft_delete_query(user.id, &payload.uuid, pool)
        .await
        .map_err(ApiError::err)?;

    info!(uuid = %payload.uuid, "delete_device_handler: device deleted");
    Ok(ApiResponse::ok(()))
}

#[instrument(skip(token, params, pool), fields(page = ?params.page, page_size = ?params.page_size, filter = ?params.filter))]
pub async fn list_devices_handler(
    token: &str,
    params: &DeviceListParams,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<DeviceListResponse>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("list_devices_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 50);

    let devices = device_list_query(user.id, page, page_size, &params.filter, pool)
        .await
        .map_err(ApiError::err)?;

    let total = device_count_query(user.id, &params.filter, pool)
        .await
        .map_err(ApiError::err)?;

    let mut items = Vec::with_capacity(devices.len());
    for device in devices {
        let location_uuid = get_location_uuid_by_id(device.location_id, pool)
            .await
            .unwrap_or_default();
        items.push(device_to_public(device, auth.user_uuid.clone(), location_uuid));
    }

    Ok(ApiResponse::ok(DeviceListResponse {
        items,
        total,
        page,
        page_size,
    }))
}

#[instrument(skip(token, input, pool), fields(device_uuid = %input.uuid))]
pub async fn update_device_handler(
    token: &str,
    input: &DeviceUpdateInput,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<DevicePublic>, ApiError> {
    let auth = validate_bearer(token)?;

    input.validate().map_err(ApiError::err)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("update_device_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let existing = device_get_by_uuid_query(user.id, &input.uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let is_activating = input.is_active == Some(true);
    let has_other_updates = input.name.is_some()
        || input.description.is_some()
        || input.location_uuid.is_some()
        || input.publish_qos.is_some()
        || input.subscribe_qos.is_some()
        || input.status_retain.is_some()
        || input.data_retain.is_some()
        || input.lwt_enabled.is_some()
        || input.lwt_qos.is_some()
        || input.lwt_retain.is_some()
        || input.heartbeat_interval.is_some()
        || input.offline_threshold.is_some();

    if !existing.is_active && has_other_updates && !is_activating {
        error!("update_device_handler: trying to update inactive device");
        return Err(ApiError::err(
            "Device is inactive. Only activation is allowed.".to_string(),
        ));
    }

    let mut db_payload = input.to_db();

    if let Some(ref loc_uuid) = input.location_uuid {
        let location = location_get_by_uuid_query(user.id, loc_uuid, pool)
            .await
            .map_err(ApiError::err)?;
        db_payload.location_id = Some(location.id);
    }

    let device = device_update_query(user.id, &input.uuid, &db_payload, pool)
        .await
        .map_err(ApiError::err)?;

    let location_uuid = get_location_uuid_by_id(device.location_id, pool)
        .await
        .unwrap_or_default();

    let public = device_to_public(device, auth.user_uuid.clone(), location_uuid);

    info!(uuid = %public.uuid, "update_device_handler: device updated");
    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, device_uuid, pool), fields(device_uuid = %device_uuid))]
pub async fn get_device_handler(
    token: &str,
    device_uuid: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<DevicePublic>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_device_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device = device_get_by_uuid_query(user.id, device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let location_uuid = get_location_uuid_by_id(device.location_id, pool)
        .await
        .unwrap_or_default();

    let public = device_to_public(device, auth.user_uuid.clone(), location_uuid);

    info!(uuid = %public.uuid, "get_device_handler: device retrieved");
    Ok(ApiResponse::ok(public))
}

#[derive(Debug, serde::Serialize)]
pub struct DeviceExistsByMacResponse {
    pub exists: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct CheckDeviceByMacForAdoptionResponse {
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_user_uuid: Option<String>,
}

#[instrument(skip(token, pool), fields(mac_address = %mac_address))]
pub async fn check_device_by_mac_for_adoption_handler(
    token: &str,
    mac_address: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<CheckDeviceByMacForAdoptionResponse>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("check_device_by_mac_for_adoption_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device = device_get_by_mac_any_user_query(mac_address, pool)
        .await
        .map_err(ApiError::err)?;

    let (exists, owner_user_uuid) = match device {
        Some(d) => {
            let owner_uuid =
                get_user_uuid_by_id(d.user_id, pool).await.map_err(ApiError::err)?;
            info!(
                mac = %mac_address,
                owner_user_uuid = %owner_uuid,
                "check_device_by_mac_for_adoption: device found in DB"
            );
            (true, Some(owner_uuid))
        }
        None => {
            debug!(mac = %mac_address, "check_device_by_mac_for_adoption: MAC not in devices table");
            (false, None)
        }
    };

    Ok(ApiResponse::ok(CheckDeviceByMacForAdoptionResponse {
        exists,
        owner_user_uuid,
    }))
}

#[instrument(skip(token, pool), fields(mac_address = %mac_address))]
pub async fn check_device_by_mac_handler(
    token: &str,
    mac_address: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<DeviceExistsByMacResponse>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("check_device_by_mac_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device = device_get_by_mac_query(user.id, mac_address, pool)
        .await
        .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(DeviceExistsByMacResponse {
        exists: device.is_some(),
    }))
}

#[instrument(skip(token, pool), fields(mac_address = %mac_address))]
pub async fn get_device_by_mac_handler(
    token: &str,
    mac_address: &str,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<Option<DevicePublic>>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_device_by_mac_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device = device_get_by_mac_query(user.id, mac_address, pool)
        .await
        .map_err(ApiError::err)?;

    let public = match device {
        Some(d) => {
            let location_uuid = get_location_uuid_by_id(d.location_id, pool)
                .await
                .map_err(ApiError::err)?;
            Some(device_to_public(d, auth.user_uuid.clone(), location_uuid))
        }
        None => None,
    };

    Ok(ApiResponse::ok(public))
}

#[instrument(skip(token, filter, pool))]
pub async fn get_device_commands_for_chart_handler(
    token: &str,
    filter: &DeviceCommandsChartFilter,
    pool: &Pool<Sqlite>,
) -> Result<ApiResponse<Vec<DeviceCommandChartPoint>>, ApiError> {
    let auth = validate_bearer(token)?;

    let user = user_get_by_uuid_query(&auth.user_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    if !user.is_active {
        error!("get_device_commands_for_chart_handler: user inactive");
        return Err(ApiError::err("Unauthorized".to_string()));
    }

    let device = device_get_by_uuid_query(user.id, &filter.device_uuid, pool)
        .await
        .map_err(ApiError::err)?;

    let commands = device_commands_for_chart_query(
        device.id,
        &filter.start_date,
        &filter.end_date,
        filter.limit,
        pool,
    )
    .await
    .map_err(ApiError::err)?;

    Ok(ApiResponse::ok(commands))
}
