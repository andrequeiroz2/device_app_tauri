use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

use crate::api::trigger::trigger_validator::{
    validate_action_config_json, validate_condition_json,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceEvent {
    SensorReading,
    DeviceCommand,
    Schedule,
}

impl SourceEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceEvent::SensorReading => "sensor_reading",
            SourceEvent::DeviceCommand => "device_command",
            SourceEvent::Schedule => "schedule",
        }
    }
}

impl std::str::FromStr for SourceEvent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sensor_reading" => Ok(SourceEvent::SensorReading),
            "device_command" => Ok(SourceEvent::DeviceCommand),
            "schedule" => Ok(SourceEvent::Schedule),
            _ => Err(format!("Invalid source_event: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Discord,
    Telegram,
    DeviceCommand,
}

impl ActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionType::Discord => "discord",
            ActionType::Telegram => "telegram",
            ActionType::DeviceCommand => "device_command",
        }
    }
}

impl std::str::FromStr for ActionType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "discord" => Ok(ActionType::Discord),
            "telegram" => Ok(ActionType::Telegram),
            "device_command" => Ok(ActionType::DeviceCommand),
            _ => Err(format!("Invalid action_type: {}", s)),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct Trigger {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub device_id: Option<i64>,
    pub name: String,
    pub source_event: String,
    pub condition_json: String,
    pub action_type: String,
    pub action_config_json: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Resultado de query com JOIN em devices (device_uuid).
#[derive(Debug, FromRow)]
pub struct TriggerWithDeviceRow {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub device_id: Option<i64>,
    pub name: String,
    pub source_event: String,
    pub condition_json: String,
    pub action_type: String,
    pub action_config_json: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub device_uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerPublic {
    pub uuid: String,
    pub device_uuid: Option<String>,
    pub name: String,
    pub source_event: String,
    pub condition_json: serde_json::Value,
    pub action_type: String,
    pub action_config_json: serde_json::Value,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TriggerCreateInput {
    pub device_uuid: Option<String>,
    pub name: String,
    pub source_event: String,
    pub condition_json: serde_json::Value,
    pub action_type: String,
    pub action_config_json: serde_json::Value,
    #[serde(default = "default_is_active")]
    pub is_active: bool,
}

fn default_is_active() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct TriggerUpdateInput {
    pub uuid: String,
    pub device_uuid: Option<Option<String>>,
    pub name: Option<String>,
    pub source_event: Option<String>,
    pub condition_json: Option<serde_json::Value>,
    pub action_type: Option<String>,
    pub action_config_json: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TriggerFilter {
    pub device_uuid: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerListParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    #[serde(default)]
    pub filter: TriggerFilter,
}

#[derive(Debug, Serialize)]
pub struct TriggerListResponse {
    pub items: Vec<TriggerPublic>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug)]
pub struct TriggerCreateDB {
    pub uuid: String,
    pub user_id: i64,
    pub device_id: Option<i64>,
    pub name: String,
    pub source_event: String,
    pub condition_json: String,
    pub action_type: String,
    pub action_config_json: String,
    pub is_active: bool,
}

#[derive(Debug, Default)]
pub struct TriggerUpdateDB {
    pub device_id: Option<Option<i64>>,
    pub name: Option<String>,
    pub source_event: Option<String>,
    pub condition_json: Option<String>,
    pub action_type: Option<String>,
    pub action_config_json: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerDeleteInput {
    pub uuid: String,
}

impl TriggerCreateInput {
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err("Name is required".to_string());
        }
        if name.len() > 255 {
            return Err("Name is too long (max 255)".to_string());
        }

        let _: SourceEvent = self.source_event.parse()?;
        let _: ActionType = self.action_type.parse()?;

        validate_condition_json(&self.source_event, &self.condition_json)?;
        validate_action_config_json(&self.action_type, &self.action_config_json)?;

        Ok(())
    }

    pub fn to_db(&self, user_id: i64, device_id: Option<i64>) -> TriggerCreateDB {
        TriggerCreateDB {
            uuid: uuid::Uuid::new_v4().to_string(),
            user_id,
            device_id,
            name: self.name.trim().to_string(),
            source_event: self.source_event.clone(),
            condition_json: self.condition_json.to_string(),
            action_type: self.action_type.clone(),
            action_config_json: self.action_config_json.to_string(),
            is_active: self.is_active,
        }
    }
}

impl TriggerUpdateInput {
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), String> {
        if self.uuid.trim().is_empty() {
            return Err("UUID is required".to_string());
        }

        let has_updates = self.device_uuid.is_some()
            || self.name.is_some()
            || self.source_event.is_some()
            || self.condition_json.is_some()
            || self.action_type.is_some()
            || self.action_config_json.is_some()
            || self.is_active.is_some();

        if !has_updates {
            return Err("At least one field must be provided for update".to_string());
        }

        if let Some(ref name) = self.name {
            if name.trim().is_empty() {
                return Err("Name cannot be empty".to_string());
            }
            if name.len() > 255 {
                return Err("Name is too long (max 255)".to_string());
            }
        }

        if let Some(ref se) = self.source_event {
            let _: SourceEvent = se.parse()?;
        }
        if let Some(ref at) = self.action_type {
            let _: ActionType = at.parse()?;
        }

        if let Some(ref cond) = self.condition_json {
            let se = self
                .source_event
                .as_ref()
                .ok_or("source_event is required when updating condition_json")?;
            validate_condition_json(se, cond)?;
        }
        if let Some(ref cfg) = self.action_config_json {
            let at = self
                .action_type
                .as_ref()
                .ok_or("action_type is required when updating action_config_json")?;
            validate_action_config_json(at, cfg)?;
        }

        Ok(())
    }
}
