//! Executor for trigger action_type `device_command`: publishes to MQTT topic `{user_uuid}/{device_uuid}/command`.

use serde_json::Value;
use sqlx::{Pool, Sqlite};
use tracing::instrument;

use crate::api::user::user_query::user_get_by_id_query;
use crate::collector::state::CollectorState;

/// Builds MQTT payload string from validated device_command action_config.
/// - If config has `command` (string): returns `{"action": "<command>"}`.
/// - If config has `command_payload` (object): returns that object as JSON string.
pub fn device_command_payload_from_config(config: &serde_json::Map<String, Value>) -> Result<String, String> {
    let has_command = config.contains_key("command");
    let has_payload = config.contains_key("command_payload");
    if has_command && has_payload {
        return Err("use either command or command_payload, not both".to_string());
    }
    if has_command {
        let cmd = config
            .get("command")
            .and_then(Value::as_str)
            .ok_or("command must be a string")?;
        return Ok(serde_json::json!({ "action": cmd }).to_string());
    }
    if has_payload {
        let pl = config
            .get("command_payload")
            .ok_or("command_payload required")?;
        if !pl.is_object() {
            return Err("command_payload must be an object".to_string());
        }
        return serde_json::to_string(pl).map_err(|e| e.to_string());
    }
    Err("command or command_payload is required".to_string())
}

/// Publishes a device command to MQTT topic `{user_uuid}/{target_device_uuid}/command`.
/// Resolves `user_id` → user uuid; payload must be the final JSON string (use `device_command_payload_from_config` if needed).
#[instrument(skip(collector_state, pool))]
pub async fn execute_device_command(
    collector_state: &CollectorState,
    pool: &Pool<Sqlite>,
    user_id: i64,
    target_device_uuid: &str,
    payload: &str,
) -> Result<(), String> {
    let user = user_get_by_id_query(user_id, pool).await?;
    let topic = format!("{}/{}/command", user.uuid, target_device_uuid);
    collector_state
        .send_publish(topic, payload.to_string())
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn device_command_payload_from_config_command_string() {
        let config = serde_json::map::Map::from_iter([(
            "command".to_string(),
            Value::String("ON".to_string()),
        )]);
        let out = device_command_payload_from_config(&config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed.get("action").and_then(|v| v.as_str()), Some("ON"));
    }

    #[test]
    fn device_command_payload_from_config_command_payload_object() {
        let config = serde_json::map::Map::from_iter([(
            "command_payload".to_string(),
            json!({ "action": "set_temp", "value": 25 }),
        )]);
        let out = device_command_payload_from_config(&config).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed.get("action").and_then(|v| v.as_str()), Some("set_temp"));
        assert_eq!(parsed.get("value").and_then(|v| v.as_i64()), Some(25));
    }

    #[test]
    fn device_command_payload_from_config_both_command_and_payload_err() {
        let config = serde_json::map::Map::from_iter([
            ("command".to_string(), Value::String("ON".to_string())),
            ("command_payload".to_string(), json!({"action":"OFF"})),
        ]);
        let result = device_command_payload_from_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("either command or command_payload"));
    }

    #[test]
    fn device_command_payload_from_config_neither_err() {
        let config = serde_json::map::Map::new();
        let result = device_command_payload_from_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("command or command_payload"));
    }

    #[test]
    fn device_command_payload_from_config_command_payload_not_object_err() {
        let config = serde_json::map::Map::from_iter([(
            "command_payload".to_string(),
            Value::String("invalid".to_string()),
        )]);
        let result = device_command_payload_from_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be an object"));
    }
}
