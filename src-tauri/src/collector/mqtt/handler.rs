use paho_mqtt::Message;
use tracing::{info, error};
use crate::collector::persistence::query::save_mqtt_message_query;

pub async fn handle_mqtt_message(message: Message, pool: &sqlx::Pool<sqlx::Sqlite>) {
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
    }
}

