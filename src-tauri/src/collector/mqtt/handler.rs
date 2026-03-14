use paho_mqtt::Message;
use tauri::Emitter;
use tracing::{info, error, warn};
use crate::api::sensor_reading::sensor_reading_query::sensor_reading_batch_insert;
use crate::collector::mqtt::data_processor::process_mqtt_data_message;
use crate::collector::mqtt::status_processor::process_mqtt_status_message;
use crate::collector::persistence::query::{
    save_mqtt_message_query, update_device_operation_status_by_uuid_query,
};
use crate::collector::topic::parse_topic_uuid;

const EVENT_DEVICE_DASHBOARD_UPDATE: &str = "device-dashboard-update";

/// Returns true if the topic suffix is relevant for the device dashboard.
fn is_dashboard_relevant_topic(topic: &str) -> bool {
    topic.ends_with("/data") || topic.ends_with("/status") || topic.ends_with("/command")
}

pub async fn handle_mqtt_message(
    message: Message,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    app: &tauri::AppHandle,
) {
    let topic = message.topic().to_string();
    let payload = String::from_utf8_lossy(message.payload()).to_string();
    let qos = message.qos();
    let retain = message.retained();

    info!(
        topic = %topic,
        qos = qos,
        retain = retain,
        payload_len = payload.len(),
        "MQTT message received"
    );

    // Save message to database
    if let Err(e) = save_mqtt_message_query(&topic, &payload, qos as i32, retain, pool).await {
        error!(error = %e, topic = %topic, "Failed to save MQTT message");
        return;
    }

    // Process /data topics → populate sensor_readings (Fase 2)
    if topic.ends_with("/data") {
        match process_mqtt_data_message(&topic, &payload, pool).await {
            Ok(processed) if !processed.readings.is_empty() => {
                if let Err(e) =
                    sensor_reading_batch_insert(processed.device_id, &processed.readings, pool).await
                {
                    warn!(error = %e, topic = %topic, "Failed to insert sensor readings");
                }
            }
            Err(e) => {
                warn!(error = %e, topic = %topic, "Failed to process MQTT data message");
            }
            _ => {}
        }
    }

    // Process /status topics → update operation_status and last_seen_at (Fase 3)
    if topic.ends_with("/status") {
        match process_mqtt_status_message(&topic, &payload) {
            Ok(processed) => {
                if let Err(e) = update_device_operation_status_by_uuid_query(
                    pool,
                    &processed.device_uuid,
                    &processed.operation_status,
                    &processed.last_seen_at,
                )
                .await
                {
                    warn!(error = %e, topic = %topic, "Failed to update device operation status");
                }
            }
            Err(e) => {
                warn!(error = %e, topic = %topic, "Failed to process MQTT status message");
            }
        }
    }

    // Emit event for dashboard real-time update when topic is relevant
    if is_dashboard_relevant_topic(&topic) {
        let (_, device_uuid) = parse_topic_uuid(&topic);
        if let Some(uuid) = device_uuid {
            let _ = app.emit(EVENT_DEVICE_DASHBOARD_UPDATE, serde_json::json!({ "device_uuid": uuid }));
        }
    }
}

