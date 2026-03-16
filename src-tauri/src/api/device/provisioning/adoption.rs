use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

use super::error::{map_provisioning_error, ProvisioningError};
use super::protocol::{DeviceConfig, DeviceInfo, DeviceProtocol};
use super::serial::SerialConnection;
use crate::api::device::device_handler::create_device_handler;
use crate::api::device::device_model::{DeviceCreateInput, DevicePublic, DeviceType};
use crate::api::icon::icon_query::icon_get_by_uuid_query;
use crate::api::user::user_query::user_get_by_uuid_query;
use crate::collector::persistence::query::get_default_broker_by_user_query;

/// Input for probing a device (connect + get_info)
#[derive(Debug, Deserialize)]
pub struct ProbeDeviceInput {
    pub port: String,
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
}

fn default_baud_rate() -> u32 {
    115200
}

/// Result of probing a device
#[derive(Debug, Serialize)]
pub struct ProbeDeviceResult {
    pub port: String,
    pub firmware_version: Option<String>,
    pub device_info: DeviceInfo,
    pub can_adopt: bool,
    pub message: Option<String>,
}

/// Input for adopting a device
#[derive(Debug, Clone, Deserialize)]
pub struct AdoptDeviceInput {
    pub port: String,
    #[serde(default = "default_baud_rate")]
    pub baud_rate: u32,
    pub name: String,
    pub location_uuid: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_uuid: Option<String>,
    #[serde(default)]
    pub broker_url: String,
    #[serde(default)]
    pub wifi_ssid: String,
    #[serde(default)]
    pub wifi_password: String,
    pub device_info: DeviceInfoInput,
}

/// Device info passed from frontend (originally from get_info)
/// Note: `model` corresponds to `boarder_type` from DeviceInfo
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceInfoInput {
    pub device_type: String,
    pub model: String,  // Maps to boarder_type from DeviceInfo
    pub mac_address: String,
    #[serde(default)]
    pub sensor_type: Option<String>,
    #[serde(default)]
    pub actuator_type: Option<String>,
    #[serde(default)]
    pub device_scale: Option<serde_json::Value>,
    #[serde(default)]
    pub parameter_ranges: Option<serde_json::Value>,
    #[serde(default)]
    pub command_spec: Option<serde_json::Value>,
    #[serde(default)]
    pub firmware_version: Option<String>,
}

/// Callback type for emitting log lines during provisioning (for real-time SerialConsole)
pub type ProvisioningLogEmitter = Arc<dyn Fn(&str) + Send + Sync>;

/// Probe a device: connect, ping, get_info.
/// Fast path first: tenta conexão direta (sem reset) — funciona quando main.py já está rodando.
/// Se falhar: faz reset via DTR, espera boot, reconecta e tenta novamente.
#[instrument(skip_all, fields(port = %input.port, baud = input.baud_rate))]
pub async fn probe_device(
    input: ProbeDeviceInput,
    emit_log: Option<ProvisioningLogEmitter>,
) -> Result<ProbeDeviceResult, String> {
    info!("probing device");

    let emit = |msg: &str| {
        if let Some(ref f) = emit_log {
            f(msg);
        }
    };

    const MAX_RETRIES: u32 = 4;
    const BACKOFF_BASE_MS: u64 = 1000;

    let mut last_error = String::new();
    let (ping_response, device_info) = 'retry: loop {
        let do_reset = !last_error.is_empty();
        if do_reset {
            emit("Opening serial connection (reset)...");
            let conn = SerialConnection::open(&input.port, input.baud_rate, false)
                .await
                .map_err(|e| map_provisioning_error(&e))?;
            drop(conn);
            emit("Waiting for device to boot...");
            tokio::time::sleep(std::time::Duration::from_secs(8)).await;
        } else {
            emit("Opening serial connection...");
        }

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay_ms = BACKOFF_BASE_MS * attempt as u64;
                emit(&format!("Retrying in {}s... (attempt {}/{})", delay_ms / 1000, attempt + 1, MAX_RETRIES));
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }

            emit(if attempt == 0 && !do_reset {
                "Sending ping..."
            } else if attempt == 0 {
                "Reconnecting..."
            } else {
                "Reconnecting (retry)..."
            });
            let conn = match SerialConnection::open(&input.port, input.baud_rate, true).await {
                Ok(c) => c,
                Err(e) => {
                    last_error = map_provisioning_error(&e);
                    if attempt + 1 >= MAX_RETRIES {
                        if !do_reset {
                            continue 'retry; // tenta com reset (last_error preenchido => do_reset=true)
                        }
                        return Err(last_error.clone());
                    }
                    continue;
                }
            };

            let mut protocol = DeviceProtocol::new(conn);

            if attempt > 0 || do_reset {
                emit("Sending ping...");
            }
            let ping_response = match protocol.ping().await {
                Ok(r) => r,
                Err(e) => {
                    last_error = map_provisioning_error(&e);
                    drop(protocol);
                    if attempt + 1 >= MAX_RETRIES {
                        if !do_reset {
                            continue 'retry; // tenta com reset
                        }
                        return Err(last_error.clone());
                    }
                    continue;
                }
            };
            emit(&format!(
                "Ping OK (version: {})",
                ping_response.version.as_deref().unwrap_or("unknown")
            ));

            debug!(version = ?ping_response.version, "device is compatible");

            emit("Requesting device info...");
            let device_info = match protocol.get_info().await {
                Ok(i) => i,
                Err(e) => {
                    last_error = map_provisioning_error(&e);
                    drop(protocol);
                    if attempt + 1 >= MAX_RETRIES {
                        if !do_reset {
                            continue 'retry;
                        }
                        return Err(last_error.clone());
                    }
                    continue;
                }
            };
            break 'retry (ping_response, device_info);
        }
    };

    emit(&format!(
        "Device: {} {} - MAC: {}",
        device_info.device_type,
        device_info.boarder_type,
        device_info.mac_address
    ));

    let can_adopt = !device_info.is_adopted();
    let message = if device_info.is_adopted() {
        Some("Device is already adopted. Reset it to adopt again.".to_string())
    } else {
        None
    };

    info!(
        mac = %device_info.mac_address,
        device_type = %device_info.device_type,
        can_adopt = can_adopt,
        "device probed successfully"
    );

    Ok(ProbeDeviceResult {
        port: input.port,
        firmware_version: ping_response.version,
        device_info,
        can_adopt,
        message,
    })
}

/// Adopt a device: set_config, reboot, create in database
#[instrument(skip(pool, token, emit_log), fields(port = %input.port, name = %input.name))]
pub async fn adopt_device(
    token: &str,
    input: AdoptDeviceInput,
    pool: &Pool<Sqlite>,
    emit_log: Option<ProvisioningLogEmitter>,
) -> Result<DevicePublic, String> {
    info!("starting device adoption");

    let emit = |msg: &str| {
        if let Some(ref f) = emit_log {
            f(msg);
        }
    };

    emit("Resolving broker configuration...");
    let device_uuid = uuid::Uuid::new_v4().to_string();
    let user_uuid = extract_user_uuid_from_token(token)?;

    let broker_url = resolve_broker_url(token, input.broker_url.as_str(), pool).await?;
    emit(&format!("Broker: {}", broker_url));

    validate_adopt_input_with_broker(&input, &broker_url)?;

    emit("Opening serial connection...");
    let conn = SerialConnection::open(&input.port, input.baud_rate, true)
        .await
        .map_err(|e| map_provisioning_error(&e))?;
    emit("Connected. Verifying device...");

    let mut protocol = DeviceProtocol::new(conn);

    protocol
        .ping()
        .await
        .map_err(|e| map_provisioning_error(&e))?;
    emit("Ping OK");

    let current_info = protocol
        .get_info()
        .await
        .map_err(|e| map_provisioning_error(&e))?;
    emit("Device info verified");

    if current_info.is_adopted() {
        warn!(mac = %current_info.mac_address, "device is already adopted");
        return Err(map_provisioning_error(&ProvisioningError::DeviceAlreadyAdopted));
    }

    if current_info.mac_address != input.device_info.mac_address {
        error!(
            expected = %input.device_info.mac_address,
            got = %current_info.mac_address,
            "MAC address mismatch"
        );
        return Err("MAC address changed. Device may have been swapped.".to_string());
    }

    let topic = format!("{}/{}", user_uuid, device_uuid);

    let config = DeviceConfig {
        user_uuid: user_uuid.clone(),
        device_uuid: device_uuid.clone(),
        device_name: input.name.clone(),
        topic,
        broker_url,
        wifi_ssid: input.wifi_ssid.clone(),
        wifi_password: input.wifi_password.clone(),
        adopted_status: 1,
        adopted_status_desc: "adopted".to_string(),
    };

    emit("Sending configuration to device...");
    debug!("sending configuration to device");

    protocol
        .set_config(config)
        .await
        .map_err(|e| map_provisioning_error(&e))?;
    emit("Configuration sent");

    info!("configuration sent, rebooting device");
    emit("Rebooting device...");

    if let Err(e) = protocol.reboot().await {
        warn!(error = %e, "reboot command may have failed (device might have disconnected)");
    }

    drop(protocol);
    emit("Device rebooting");

    emit("Creating device in database...");
    debug!("creating device in database");

    let device_type = parse_device_type(&input.device_info.device_type)?;

    let icon_id = if let Some(ref icon_uuid) = input.icon_uuid {
        let icon = icon_get_by_uuid_query(icon_uuid, pool)
            .await
            .map_err(|e| format!("Invalid icon: {}", e))?;
        Some(icon.id)
    } else {
        None
    };

    let create_input = DeviceCreateInput {
        name: input.name,
        location_uuid: input.location_uuid,
        description: input.description,
        device_type,
        model: input.device_info.model,
        mac_address: input.device_info.mac_address,
        firmware_version: input.device_info.firmware_version,
        sensor_type: input.device_info.sensor_type,
        actuator_type: input.device_info.actuator_type,
        device_scale: input.device_info.device_scale,
        parameter_ranges: input.device_info.parameter_ranges,
        command_spec: input.device_info.command_spec,
        icon_id,
    };

    let response = create_device_handler(token, &create_input, pool)
        .await
        .map_err(|e| e.message)?;

    let device = response.data.ok_or("Failed to create device")?;
    emit("Adoption complete!");

    info!(
        uuid = %device.uuid,
        mac = %device.mac_address,
        "device adopted successfully"
    );

    Ok(device)
}

fn validate_adopt_input_with_broker(
    input: &AdoptDeviceInput,
    broker_url: &str,
) -> Result<(), String> {
    if input.name.trim().is_empty() {
        return Err("Device name is required".to_string());
    }

    if input.location_uuid.trim().is_empty() {
        return Err("Location is required".to_string());
    }

    if broker_url.trim().is_empty() {
        return Err("Broker URL is required. Set a default MQTT broker first.".to_string());
    }

    if input.device_info.mac_address.trim().is_empty() {
        return Err("MAC address is required".to_string());
    }

    Ok(())
}

fn parse_device_type(dtype: &str) -> Result<DeviceType, String> {
    match dtype.to_lowercase().as_str() {
        "sensor" => Ok(DeviceType::Sensor),
        "actuator" => Ok(DeviceType::Actuator),
        _ => Err(format!("Invalid device type: {}", dtype)),
    }
}

fn extract_user_uuid_from_token(token: &str) -> Result<String, String> {
    use crate::api::auth::auth_validator::validate_bearer;

    let claims = validate_bearer(token).map_err(|e| format!("Invalid token: {}", e.message))?;

    Ok(claims.user_uuid)
}

/// Broker info for display in adoption wizard (read-only fields)
#[derive(Debug, Serialize)]
pub struct DefaultBrokerInfo {
    pub host: String,
    pub port: i32,
    pub use_tls: bool,
    pub broker_url: String,
}

/// Get default broker info for the current user (for adoption wizard display)
#[instrument(skip(pool, token))]
pub async fn get_default_broker_for_adoption(
    token: &str,
    pool: &Pool<Sqlite>,
) -> Result<Option<DefaultBrokerInfo>, String> {
    let broker = get_broker_for_user(token, pool).await?;
    Ok(broker.map(|b| {
        let scheme = if b.use_tls { "mqtts" } else { "mqtt" };
        let broker_url = format!("{}://{}:{}", scheme, b.host, b.port);
        DefaultBrokerInfo {
            host: b.host,
            port: b.port,
            use_tls: b.use_tls,
            broker_url,
        }
    }))
}

async fn get_broker_for_user(
    token: &str,
    pool: &Pool<Sqlite>,
) -> Result<Option<crate::collector::model::MqttBroker>, String> {
    let user_uuid = extract_user_uuid_from_token(token)?;
    let user = user_get_by_uuid_query(&user_uuid, pool)
        .await
        .map_err(|e| format!("User not found: {}", e))?;
    let broker: Option<crate::collector::model::MqttBroker> =
        get_default_broker_by_user_query(pool, user.id)
            .await
            .map_err(|e| format!("Failed to get broker: {}", e))?;
    Ok(broker)
}

/// Resolve broker URL: use provided value or fetch from user's default broker
async fn resolve_broker_url(
    token: &str,
    provided: &str,
    pool: &Pool<Sqlite>,
) -> Result<String, String> {
    if !provided.trim().is_empty() {
        return Ok(provided.trim().to_string());
    }

    let user_uuid = extract_user_uuid_from_token(token)?;
    let user = user_get_by_uuid_query(&user_uuid, pool)
        .await
        .map_err(|e| format!("User not found: {}", e))?;

    let broker: crate::collector::model::MqttBroker = get_default_broker_by_user_query(pool, user.id)
        .await
        .map_err(|e| format!("Failed to get default broker: {}", e))?
        .ok_or_else(|| {
            "No default MQTT broker configured. Create and set a broker as default first.".to_string()
        })?;

    let scheme = if broker.use_tls { "mqtts" } else { "mqtt" };
    let url = format!("{}://{}:{}", scheme, broker.host, broker.port);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_device_type() {
        assert!(matches!(parse_device_type("sensor"), Ok(DeviceType::Sensor)));
        assert!(matches!(parse_device_type("Sensor"), Ok(DeviceType::Sensor)));
        assert!(matches!(parse_device_type("SENSOR"), Ok(DeviceType::Sensor)));
        assert!(matches!(
            parse_device_type("actuator"),
            Ok(DeviceType::Actuator)
        ));
        assert!(parse_device_type("invalid").is_err());
    }

    #[test]
    fn test_validate_adopt_input_with_broker() {
        let valid = AdoptDeviceInput {
            port: "/dev/ttyUSB0".to_string(),
            baud_rate: 115200,
            name: "Test Device".to_string(),
            location_uuid: "location-uuid".to_string(),
            description: None,
            icon_uuid: None,
            broker_url: "mqtt://localhost:1883".to_string(),
            wifi_ssid: "MyWifi".to_string(),
            wifi_password: "secret".to_string(),
            device_info: DeviceInfoInput {
                device_type: "sensor".to_string(),
                model: "ESP32".to_string(),
                mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
                sensor_type: None,
                actuator_type: None,
                device_scale: None,
                parameter_ranges: None,
                command_spec: None,
                firmware_version: None,
            },
        };
        assert!(validate_adopt_input_with_broker(&valid, "mqtt://localhost:1883").is_ok());

        let empty_name = AdoptDeviceInput {
            name: "".to_string(),
            ..valid.clone()
        };
        assert!(validate_adopt_input_with_broker(&empty_name, "mqtt://localhost:1883").is_err());
    }
}
