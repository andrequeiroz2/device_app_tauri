use sqlx::{Pool, Sqlite};
use tracing::{error, instrument};
use crate::collector::model::MqttBrokerRow;
use crate::collector::topic::parse_topic_uuid;

#[instrument(skip(pool))]
pub async fn get_default_broker_query(pool: &Pool<Sqlite>) -> Result<Option<crate::collector::model::MqttBroker>, String> {
    let result = sqlx::query_as::<_, MqttBrokerRow>(
        r#"
        SELECT 
            id, uuid, user_id, name, description, host, port,
            username, password, use_tls, ca_certificate_path,
            client_certificate_path, client_key_path, insecure_tls,
            client_id, keep_alive_interval, clean_session,
            connection_timeout_secs, operation_timeout_secs,
            last_will_topic, last_will_message, last_will_qos,
            last_will_retain, is_active, is_connected, is_default
        FROM mqtt_brokers
        WHERE is_active = 1 AND is_default = 1
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to get default broker");
        format!("Failed to get default broker: {}", e)
    })?;

    Ok(result.map(|row| row.into()))
}

/// Gets the default broker for a specific user
/// Returns the broker with is_default = 1 for the given user_id
#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn get_default_broker_by_user_query(
    pool: &Pool<Sqlite>,
    user_id: i64,
) -> Result<Option<crate::collector::model::MqttBroker>, String> {
    let result = sqlx::query_as::<_, MqttBrokerRow>(
        r#"
        SELECT 
            id, uuid, user_id, name, description, host, port,
            username, password, use_tls, ca_certificate_path,
            client_certificate_path, client_key_path, insecure_tls,
            client_id, keep_alive_interval, clean_session,
            connection_timeout_secs, operation_timeout_secs,
            last_will_topic, last_will_message, last_will_qos,
            last_will_retain, is_active, is_connected, is_default
        FROM mqtt_brokers
        WHERE is_active = 1 AND is_default = 1 AND user_id = ?1
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, user_id = user_id, "Failed to get default broker by user");
        format!("Failed to get default broker for user {}: {}", user_id, e)
    })?;

    Ok(result.map(|row| row.into()))
}

#[instrument(skip(pool), fields(topic = %topic))]
pub async fn save_mqtt_message_query(
    topic: &str,
    payload: &str,
    qos: i32,
    retain: bool,
    pool: &Pool<Sqlite>,
) -> Result<(), String> {
    // Parse topic to extract broker_uuid and device_uuid
    // Expected format: {broker_uuid}/{device_uuid}/...
    let (broker_uuid, device_uuid) = parse_topic_uuid(topic);
    
    sqlx::query(
        r#"
        INSERT INTO mqtt_messages (topic, broker_uuid, device_uuid, payload, qos, retain)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(topic)
    .bind(broker_uuid)
    .bind(device_uuid)
    .bind(payload)
    .bind(qos)
    .bind(retain)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, topic = %topic, "Failed to save MQTT message");
        format!("Failed to save MQTT message: {}", e)
    })?;

    Ok(())
}

#[instrument(skip(pool))]
pub async fn get_mqtt_messages_query(
    topic: Option<&str>,
    limit: Option<i64>,
    pool: &Pool<Sqlite>,
) -> Result<Vec<crate::collector::model::MqttMessage>, String> {
    use crate::collector::model::MqttMessage;
    
    let limit = limit.unwrap_or(100).min(1000); // Max 1000
    
    let messages = if let Some(topic_filter) = topic {
        let pattern = format!("%{}%", topic_filter);
        sqlx::query_as::<_, MqttMessage>(
            "SELECT id, topic, broker_uuid, device_uuid, payload, qos, retain, received_at 
             FROM mqtt_messages 
             WHERE topic LIKE ? 
             ORDER BY received_at DESC 
             LIMIT ?"
        )
        .bind(pattern)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, MqttMessage>(
            "SELECT id, topic, broker_uuid, device_uuid, payload, qos, retain, received_at 
             FROM mqtt_messages 
             ORDER BY received_at DESC 
             LIMIT ?"
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| {
        error!(error = %e, "Failed to get MQTT messages");
        format!("Failed to get MQTT messages: {}", e)
    })?;
    
    Ok(messages)
}

#[instrument(skip(pool))]
pub async fn count_mqtt_messages_query(pool: &Pool<Sqlite>) -> Result<i64, String> {
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mqtt_messages"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to count MQTT messages");
        format!("Failed to count MQTT messages: {}", e)
    })?;
    
    Ok(result.0)
}

#[instrument(skip(pool))]
pub async fn get_last_message_at_query(pool: &Pool<Sqlite>) -> Result<Option<String>, String> {
    let result: Option<(String,)> = sqlx::query_as(
        "SELECT received_at FROM mqtt_messages ORDER BY received_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to get last message timestamp");
        format!("Failed to get last message timestamp: {}", e)
    })?;
    
    Ok(result.map(|r| r.0))
}

#[instrument(skip(pool))]
pub async fn get_default_broker_status_query(pool: &Pool<Sqlite>) -> Result<Option<(String, bool)>, String> {
    let result: Option<(String, bool)> = sqlx::query_as(
        "SELECT name, is_connected FROM mqtt_brokers WHERE is_active = 1 AND is_default = 1 LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to get broker status");
        format!("Failed to get broker status: {}", e)
    })?;
    
    Ok(result)
}

/// Get MQTT messages filtered by broker UUID and optionally device UUID
#[instrument(skip(pool), fields(broker_uuid = %broker_uuid))]
pub async fn get_mqtt_messages_by_broker_query(
    broker_uuid: &str,
    device_uuid: Option<&str>,
    limit: Option<i64>,
    pool: &Pool<Sqlite>,
) -> Result<Vec<crate::collector::model::MqttMessage>, String> {
    use crate::collector::model::MqttMessage;
    
    let limit = limit.unwrap_or(100).min(1000);
    
    let messages = if let Some(device) = device_uuid {
        sqlx::query_as::<_, MqttMessage>(
            "SELECT id, topic, broker_uuid, device_uuid, payload, qos, retain, received_at 
             FROM mqtt_messages 
             WHERE broker_uuid = ? AND device_uuid = ?
             ORDER BY received_at DESC 
             LIMIT ?"
        )
        .bind(broker_uuid)
        .bind(device)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, MqttMessage>(
            "SELECT id, topic, broker_uuid, device_uuid, payload, qos, retain, received_at 
             FROM mqtt_messages 
             WHERE broker_uuid = ?
             ORDER BY received_at DESC 
             LIMIT ?"
        )
        .bind(broker_uuid)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| {
        error!(error = %e, broker_uuid = %broker_uuid, "Failed to get MQTT messages by broker");
        format!("Failed to get MQTT messages by broker: {}", e)
    })?;
    
    Ok(messages)
}

/// Get MQTT messages filtered by device UUID
#[instrument(skip(pool), fields(device_uuid = %device_uuid))]
pub async fn get_mqtt_messages_by_device_query(
    device_uuid: &str,
    limit: Option<i64>,
    pool: &Pool<Sqlite>,
) -> Result<Vec<crate::collector::model::MqttMessage>, String> {
    use crate::collector::model::MqttMessage;
    
    let limit = limit.unwrap_or(100).min(1000);
    
    let messages = sqlx::query_as::<_, MqttMessage>(
        "SELECT id, topic, broker_uuid, device_uuid, payload, qos, retain, received_at 
         FROM mqtt_messages 
         WHERE device_uuid = ?
         ORDER BY received_at DESC 
         LIMIT ?"
    )
    .bind(device_uuid)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        error!(error = %e, device_uuid = %device_uuid, "Failed to get MQTT messages by device");
        format!("Failed to get MQTT messages by device: {}", e)
    })?;
    
    Ok(messages)
}


