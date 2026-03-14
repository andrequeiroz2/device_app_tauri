//! Fase 1: Processa mensagens MQTT em tópicos `/data` e monta leituras prontas para insert.
//!
//! Recebe topic + payload, extrai measurements numéricos, resolve device_id e device_scale,
//! e retorna `Vec<(measurement, value, scale, recorded_at)>`.

use serde_json::Value;
use sqlx::{Pool, Sqlite};

use crate::collector::persistence::query::get_device_id_and_scale_by_uuid_query;
use crate::collector::topic::parse_topic_uuid;

/// Entrada para inserção em sensor_readings: (measurement, value, scale, recorded_at).
pub type SensorReadingTuple = (String, f64, String, String);

/// Resultado do processamento: device_id e lista de leituras.
#[derive(Debug)]
pub struct ProcessedDataReadings {
    pub device_id: i64,
    pub readings: Vec<SensorReadingTuple>,
}

/// Extrai measurements numéricos do payload JSON.
/// Suporta timestamp opcional (ISO8601) para recorded_at.
/// Ignora campos não numéricos. JSON inválido → Err (4.3)
pub(crate) fn parse_data_payload(
    payload: &str,
    recorded_at_fallback: &str,
) -> Result<Vec<(String, f64, String)>, String> {
    let root: Value = serde_json::from_str(payload)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let obj = root.as_object().ok_or("Expected JSON object")?;

    let mut recorded_at = recorded_at_fallback.to_string();
    if let Some(ts) = obj.get("timestamp") {
        if let Some(s) = ts.as_str() {
            recorded_at = s.to_string();
        }
    }

    let mut measurements = Vec::new();
    for (key, val) in obj {
        if key == "timestamp" {
            continue;
        }
        if let Some(n) = val.as_f64() {
            measurements.push((key.clone(), n, recorded_at.clone()));
        } else if val.as_i64().is_some() {
            let n = val.as_i64().unwrap() as f64;
            measurements.push((key.clone(), n, recorded_at.clone()));
        }
    }

    Ok(measurements)
}

/// Mapeia measurement → scale a partir do device_scale JSON.
/// Formato: [["temperature","C"],["humidity","%"]]
/// Device sem device_scale ou measurement não mapeado → "" (4.2)
pub(crate) fn get_scale_for_measurement(device_scale: Option<&str>, measurement: &str) -> String {
    let scale_json = match device_scale {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };

    let arr: Vec<Vec<String>> = match serde_json::from_str(scale_json) {
        Ok(a) => a,
        Err(_) => return String::new(),
    };

    for pair in arr {
        if pair.len() >= 2 && pair[0] == measurement {
            return pair[1].clone();
        }
    }

    String::new()
}

/// Processa mensagem MQTT em tópico /data.
/// Retorna device_id e leituras prontas para sensor_reading_batch_insert.
pub async fn process_mqtt_data_message(
    topic: &str,
    payload: &str,
    pool: &Pool<Sqlite>,
) -> Result<ProcessedDataReadings, String> {
    if !topic.ends_with("/data") {
        return Err("Topic must end with /data".to_string());
    }

    let (_, device_uuid) = parse_topic_uuid(topic);
    let device_uuid = device_uuid.ok_or("Could not parse device_uuid from topic")?;

    let (device_id, device_scale) = get_device_id_and_scale_by_uuid_query(pool, &device_uuid).await?;

    let now_iso = chrono::Utc::now().to_rfc3339();
    let raw = parse_data_payload(payload, &now_iso)?;

    if raw.is_empty() {
        return Err("No numeric measurements in payload".to_string());
    }

    let readings: Vec<SensorReadingTuple> = raw
        .into_iter()
        .map(|(measurement, value, recorded_at)| {
            let scale = get_scale_for_measurement(device_scale.as_deref(), &measurement);
            (measurement, value, scale, recorded_at)
        })
        .collect();

    Ok(ProcessedDataReadings {
        device_id,
        readings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_data_payload_valid_numerics() {
        let r = parse_data_payload(r#"{"temperature": 25.5, "humidity": 60}"#, "2025-01-01T00:00:00Z");
        assert!(r.is_ok());
        let m = r.unwrap();
        assert_eq!(m.len(), 2);
        assert!(m.iter().any(|(k, v, _)| k == "temperature" && (*v - 25.5).abs() < 0.01));
        assert!(m.iter().any(|(k, v, _)| k == "humidity" && (*v - 60.0).abs() < 0.01));
    }

    #[test]
    fn parse_data_payload_ignores_non_numeric() {
        let r = parse_data_payload(
            r#"{"temperature": 25, "label": "sala", "active": true}"#,
            "2025-01-01T00:00:00Z",
        );
        assert!(r.is_ok());
        let m = r.unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].0, "temperature");
    }

    #[test]
    fn parse_data_payload_with_timestamp() {
        let r = parse_data_payload(
            r#"{"temperature": 20, "timestamp": "2025-02-28T12:00:00Z"}"#,
            "fallback",
        );
        assert!(r.is_ok());
        let m = r.unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].2, "2025-02-28T12:00:00Z");
    }

    #[test]
    fn parse_data_payload_invalid_json_returns_err() {
        let r = parse_data_payload("not json", "fallback");
        assert!(r.is_err());
    }

    #[test]
    fn parse_data_payload_non_object_returns_err() {
        let r = parse_data_payload("[1,2,3]", "fallback");
        assert!(r.is_err());
    }

    #[test]
    fn get_scale_device_without_device_scale_returns_empty() {
        assert_eq!(get_scale_for_measurement(None, "temperature"), "");
        assert_eq!(get_scale_for_measurement(Some(""), "temperature"), "");
    }

    #[test]
    fn get_scale_measurement_not_in_map_returns_empty() {
        let scale = r#"[["temperature","C"],["humidity","%"]]"#;
        assert_eq!(get_scale_for_measurement(Some(scale), "pressure"), "");
    }

    #[test]
    fn get_scale_valid_mapping() {
        let scale = r#"[["temperature","C"],["humidity","%"]]"#;
        assert_eq!(get_scale_for_measurement(Some(scale), "temperature"), "C");
        assert_eq!(get_scale_for_measurement(Some(scale), "humidity"), "%");
    }

    #[test]
    fn get_scale_invalid_json_returns_empty() {
        assert_eq!(get_scale_for_measurement(Some("invalid"), "temperature"), "");
    }
}
