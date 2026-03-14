//! Fase 3: Processa mensagens MQTT em tópicos `/status` e retorna dados para atualizar device.
//!
//! Payload esperado: `{"state":"online"}` ou `{"state":"offline"}`.
//! Mapeamento: "online" → "online", "offline" ou ausência → "offline".

use serde_json::Value;

use crate::collector::topic::parse_topic_uuid;

/// Parse do payload de status. Retorna "online" ou "offline".
fn parse_status_payload(payload: &str) -> &'static str {
    let root: Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(_) => return "offline",
    };

    let state = root
        .as_object()
        .and_then(|o| o.get("state"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if state.eq_ignore_ascii_case("online") {
        "online"
    } else {
        "offline"
    }
}

/// Resultado do processamento de mensagem /status.
#[derive(Debug)]
pub struct ProcessedStatusMessage {
    pub device_uuid: String,
    pub operation_status: String,
    pub last_seen_at: String,
}

/// Processa mensagem MQTT em tópico /status.
/// Retorna dados para atualizar operation_status e last_seen_at do device.
pub fn process_mqtt_status_message(
    topic: &str,
    payload: &str,
) -> Result<ProcessedStatusMessage, String> {
    if !topic.ends_with("/status") {
        return Err("Topic must end with /status".to_string());
    }

    let (_, device_uuid) = parse_topic_uuid(topic);
    let device_uuid = device_uuid.ok_or("Could not parse device_uuid from topic")?;

    let operation_status = parse_status_payload(payload).to_string();
    let last_seen_at = chrono::Utc::now().to_rfc3339();

    Ok(ProcessedStatusMessage {
        device_uuid,
        operation_status,
        last_seen_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_status_online() {
        let r = process_mqtt_status_message("broker-uuid/device-uuid/status", r#"{"state":"online"}"#);
        assert!(r.is_ok());
        assert_eq!(r.unwrap().operation_status, "online");
    }

    #[test]
    fn process_status_offline() {
        let r = process_mqtt_status_message("broker-uuid/device-uuid/status", r#"{"state":"offline"}"#);
        assert!(r.is_ok());
        assert_eq!(r.unwrap().operation_status, "offline");
    }

    #[test]
    fn process_status_malformed_json_uses_offline() {
        let r = process_mqtt_status_message("broker-uuid/device-uuid/status", "not json");
        assert!(r.is_ok());
        assert_eq!(r.unwrap().operation_status, "offline");
    }

    #[test]
    fn process_status_empty_object_uses_offline() {
        let r = process_mqtt_status_message("broker-uuid/device-uuid/status", "{}");
        assert!(r.is_ok());
        assert_eq!(r.unwrap().operation_status, "offline");
    }

    #[test]
    fn process_status_wrong_topic_returns_err() {
        let r = process_mqtt_status_message("broker-uuid/device-uuid/data", r#"{"state":"online"}"#);
        assert!(r.is_err());
    }
}
