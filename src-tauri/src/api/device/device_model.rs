use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Actuator,
    Sensor,
}

impl DeviceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceType::Actuator => "actuator",
            DeviceType::Sensor => "sensor",
        }
    }
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    Online,
    Offline,
}

impl OperationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationStatus::Online => "online",
            OperationStatus::Offline => "offline",
        }
    }
}

impl std::fmt::Display for OperationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, FromRow)]
pub struct Device {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub location_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub device_type: String,
    pub model: String,
    pub firmware_version: Option<String>,
    pub mac_address: String,
    pub sensor_type: Option<String>,
    pub actuator_type: Option<String>,
    pub device_scale: Option<String>,
    pub adopted_at: Option<String>,
    pub operation_status: Option<String>,
    pub last_seen_at: Option<String>,
    pub ip_address: Option<String>,
    pub publish_qos: i32,
    pub subscribe_qos: i32,
    pub status_retain: bool,
    pub data_retain: bool,
    pub lwt_enabled: bool,
    pub lwt_message: Option<String>,
    pub lwt_qos: i32,
    pub lwt_retain: bool,
    pub heartbeat_interval: i32,
    pub offline_threshold: i32,
    pub last_command: Option<String>,
    pub last_command_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DevicePublic {
    pub uuid: String,
    pub user_uuid: String,
    pub location_uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub device_type: DeviceType,
    pub model: String,
    pub firmware_version: Option<String>,
    pub mac_address: String,
    pub sensor_type: Option<String>,
    pub actuator_type: Option<String>,
    pub device_scale: Option<serde_json::Value>,
    pub adopted_at: Option<String>,
    pub operation_status: Option<OperationStatus>,
    pub last_seen_at: Option<String>,
    pub ip_address: Option<String>,
    pub publish_qos: i32,
    pub subscribe_qos: i32,
    pub status_retain: bool,
    pub data_retain: bool,
    pub lwt_enabled: bool,
    pub lwt_message: Option<String>,
    pub lwt_qos: i32,
    pub lwt_retain: bool,
    pub heartbeat_interval: i32,
    pub offline_threshold: i32,
    pub last_command: Option<String>,
    pub last_command_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceCreateInput {
    pub name: String,
    pub location_uuid: String,
    pub description: Option<String>,
    pub device_type: DeviceType,
    pub model: String,
    pub mac_address: String,
    pub firmware_version: Option<String>,
    pub sensor_type: Option<String>,
    pub actuator_type: Option<String>,
    pub device_scale: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceUpdateInput {
    pub uuid: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub location_uuid: Option<String>,
    pub publish_qos: Option<i32>,
    pub subscribe_qos: Option<i32>,
    pub status_retain: Option<bool>,
    pub data_retain: Option<bool>,
    pub lwt_enabled: Option<bool>,
    pub lwt_qos: Option<i32>,
    pub lwt_retain: Option<bool>,
    pub heartbeat_interval: Option<i32>,
    pub offline_threshold: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatusFilter {
    All,
    Online,
    Offline,
}

impl Default for OperationStatusFilter {
    fn default() -> Self {
        OperationStatusFilter::All
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceTypeFilter {
    All,
    Actuator,
    Sensor,
}

impl Default for DeviceTypeFilter {
    fn default() -> Self {
        DeviceTypeFilter::All
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IsActiveFilter {
    All,
    Active,
    Inactive,
}

impl Default for IsActiveFilter {
    fn default() -> Self {
        IsActiveFilter::Active
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DeviceFilter {
    #[serde(default)]
    pub is_active: IsActiveFilter,
    #[serde(default)]
    pub operation_status: OperationStatusFilter,
    #[serde(default)]
    pub device_type: DeviceTypeFilter,
    pub location_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceListParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    #[serde(default)]
    pub filter: DeviceFilter,
}

#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub items: Vec<DevicePublic>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct DeviceCommandChartPoint {
    pub command: String,
    pub sent_at: String,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceCommandsChartFilter {
    pub device_uuid: String,
    pub start_date: String,
    pub end_date: String,
    pub limit: Option<i64>,
}

#[derive(Debug)]
pub struct DeviceCreateDB {
    pub uuid: String,
    pub user_id: i64,
    pub location_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub device_type: String,
    pub model: String,
    pub mac_address: String,
    pub firmware_version: Option<String>,
    pub sensor_type: Option<String>,
    pub actuator_type: Option<String>,
    pub device_scale: Option<String>,
}

#[derive(Debug, Default)]
pub struct DeviceUpdateDB {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location_id: Option<i64>,
    pub publish_qos: Option<i32>,
    pub subscribe_qos: Option<i32>,
    pub status_retain: Option<bool>,
    pub data_retain: Option<bool>,
    pub lwt_enabled: Option<bool>,
    pub lwt_qos: Option<i32>,
    pub lwt_retain: Option<bool>,
    pub heartbeat_interval: Option<i32>,
    pub offline_threshold: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, FromRow)]
pub struct DeviceCommand {
    pub id: i64,
    pub device_id: i64,
    pub command: String,
    pub source: String,
    pub sent_at: String,
    pub ack_at: Option<String>,
    pub response_ms: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct DeviceCommandPublic {
    pub id: i64,
    pub device_uuid: String,
    pub command: String,
    pub source: String,
    pub sent_at: String,
    pub ack_at: Option<String>,
    pub response_ms: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DeviceCommandFilter {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub command: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceCommandListParams {
    pub device_uuid: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    #[serde(default)]
    pub filter: DeviceCommandFilter,
}

#[derive(Debug, Serialize)]
pub struct DeviceCommandListResponse {
    pub items: Vec<DeviceCommandPublic>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DeviceCommandDailyStats {
    pub date: String,
    pub command: String,
    pub count: i64,
    pub avg_response_ms: Option<f64>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DeviceCommandSummary {
    pub total_commands: i64,
    pub on_count: i64,
    pub off_count: i64,
    pub avg_response_ms: Option<f64>,
    pub failed_count: i64,
}

impl DeviceCreateInput {
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();

        if name.is_empty() {
            return Err("Name is required".to_string());
        }
        if name.len() > 255 {
            return Err("Name is too long (max 255)".to_string());
        }
        if self.location_uuid.trim().is_empty() {
            return Err("Location UUID is required".to_string());
        }
        if self.mac_address.trim().is_empty() {
            return Err("MAC address is required".to_string());
        }
        if self.model.trim().is_empty() {
            return Err("Model is required".to_string());
        }

        Ok(())
    }

    pub fn to_db(&self, user_id: i64, location_id: i64) -> DeviceCreateDB {
        DeviceCreateDB {
            uuid: uuid::Uuid::new_v4().to_string(),
            user_id,
            location_id,
            name: self.name.trim().to_string(),
            description: self.description.as_ref().map(|d| d.trim().to_string()),
            device_type: self.device_type.as_str().to_string(),
            model: self.model.trim().to_string(),
            mac_address: self.mac_address.trim().to_uppercase(),
            firmware_version: self.firmware_version.as_ref().map(|v| v.trim().to_string()),
            sensor_type: self.sensor_type.as_ref().map(|s| s.trim().to_string()),
            actuator_type: self.actuator_type.as_ref().map(|a| a.trim().to_string()),
            device_scale: self.device_scale.as_ref().map(|s| s.to_string()),
        }
    }
}

impl DeviceUpdateInput {
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), String> {
        let has_updates = self.name.is_some()
            || self.description.is_some()
            || self.location_uuid.is_some()
            || self.publish_qos.is_some()
            || self.subscribe_qos.is_some()
            || self.status_retain.is_some()
            || self.data_retain.is_some()
            || self.lwt_enabled.is_some()
            || self.lwt_qos.is_some()
            || self.lwt_retain.is_some()
            || self.heartbeat_interval.is_some()
            || self.offline_threshold.is_some()
            || self.is_active.is_some();

        if !has_updates {
            return Err("At least one field must be provided for update".to_string());
        }

        if self.uuid.trim().is_empty() {
            return Err("UUID is required".to_string());
        }

        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("Name cannot be empty".to_string());
            }
            if name.len() > 255 {
                return Err("Name is too long (max 255)".to_string());
            }
        }

        if let Some(qos) = self.publish_qos {
            if !(0..=2).contains(&qos) {
                return Err("Publish QoS must be 0, 1, or 2".to_string());
            }
        }

        if let Some(qos) = self.subscribe_qos {
            if !(0..=2).contains(&qos) {
                return Err("Subscribe QoS must be 0, 1, or 2".to_string());
            }
        }

        if let Some(qos) = self.lwt_qos {
            if !(0..=2).contains(&qos) {
                return Err("LWT QoS must be 0, 1, or 2".to_string());
            }
        }

        if let Some(interval) = self.heartbeat_interval {
            if interval < 1 {
                return Err("Heartbeat interval must be at least 1 second".to_string());
            }
        }

        if let Some(threshold) = self.offline_threshold {
            if threshold < 1 {
                return Err("Offline threshold must be at least 1 second".to_string());
            }
        }

        Ok(())
    }

    pub fn to_db(&self) -> DeviceUpdateDB {
        DeviceUpdateDB {
            name: self.name.as_ref().map(|n| n.trim().to_string()),
            description: self.description.as_ref().map(|d| d.trim().to_string()),
            location_id: None, // Will be resolved from location_uuid in handler
            publish_qos: self.publish_qos,
            subscribe_qos: self.subscribe_qos,
            status_retain: self.status_retain,
            data_retain: self.data_retain,
            lwt_enabled: self.lwt_enabled,
            lwt_qos: self.lwt_qos,
            lwt_retain: self.lwt_retain,
            heartbeat_interval: self.heartbeat_interval,
            offline_threshold: self.offline_threshold,
            is_active: self.is_active,
        }
    }
}

pub fn parse_device_type(s: &str) -> DeviceType {
    match s {
        "actuator" => DeviceType::Actuator,
        "sensor" => DeviceType::Sensor,
        _ => DeviceType::Sensor,
    }
}

pub fn parse_operation_status(s: Option<&str>) -> Option<OperationStatus> {
    match s {
        Some("online") => Some(OperationStatus::Online),
        Some("offline") => Some(OperationStatus::Offline),
        _ => None,
    }
}
