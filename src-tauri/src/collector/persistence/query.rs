use serde::{Deserialize, Serialize};
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

/// Gets a broker by UUID for a specific user
#[instrument(skip(pool), fields(broker_uuid = %broker_uuid, user_id = user_id))]
pub async fn get_broker_by_uuid_and_user_query(
    pool: &Pool<Sqlite>,
    broker_uuid: &str,
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
        WHERE is_active = 1 AND uuid = ?1 AND user_id = ?2
        LIMIT 1
        "#,
    )
    .bind(broker_uuid)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(
            error = %e,
            broker_uuid = broker_uuid,
            user_id = user_id,
            "Failed to get broker by uuid and user"
        );
        format!(
            "Failed to get broker {} for user {}: {}",
            broker_uuid, user_id, e
        )
    })?;

    Ok(result.map(|row| row.into()))
}

/// Sets a broker as default for the user (unsets others first)
#[instrument(skip(pool), fields(broker_uuid = %broker_uuid, user_id = user_id))]
pub async fn set_broker_as_default_query(
    pool: &Pool<Sqlite>,
    user_id: i64,
    broker_uuid: &str,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE mqtt_brokers SET is_default = 0
        WHERE user_id = ?1 AND uuid != ?2
        "#,
    )
    .bind(user_id)
    .bind(broker_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to unset other defaults");
        format!("Failed to set broker default: {}", e)
    })?;

    sqlx::query(
        r#"
        UPDATE mqtt_brokers SET is_default = 1
        WHERE user_id = ?1 AND uuid = ?2
        "#,
    )
    .bind(user_id)
    .bind(broker_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to set broker as default");
        format!("Failed to set broker default: {}", e)
    })?;

    Ok(())
}

/// Gets device id and device_scale by uuid (no user filter - for collector context).
/// Returns (device_id, device_scale_json).
#[instrument(skip(pool), fields(device_uuid = %device_uuid))]
pub async fn get_device_id_and_scale_by_uuid_query(
    pool: &Pool<Sqlite>,
    device_uuid: &str,
) -> Result<(i64, Option<String>), String> {
    let row: Option<(i64, Option<String>)> = sqlx::query_as(
        r#"SELECT id, device_scale FROM devices WHERE uuid = ?1 AND is_active = 1"#,
    )
    .bind(device_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, device_uuid = %device_uuid, "Failed to get device by uuid");
        format!("Failed to get device: {}", e)
    })?;

    row.ok_or_else(|| "Device not found".to_string())
}

/// Gets device name by id (for trigger notifications in collector context).
#[instrument(skip(pool), fields(device_id = device_id))]
pub async fn get_device_name_by_id_query(
    pool: &Pool<Sqlite>,
    device_id: i64,
) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"SELECT name FROM devices WHERE id = ?1 AND is_active = 1"#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, device_id = device_id, "Failed to get device name");
        format!("Failed to get device name: {}", e)
    })?;
    Ok(row.map(|r| r.0))
}

/// Gets the location name for a device (collector trigger-notification context).
#[instrument(skip(pool), fields(device_id = device_id))]
pub async fn get_location_name_by_device_id_query(
    pool: &Pool<Sqlite>,
    device_id: i64,
) -> Result<Option<String>, String> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT l.name
        FROM devices d
        LEFT JOIN locations l ON d.location_id = l.id
        WHERE d.id = ?1 AND d.is_active = 1
        "#,
    )
    .bind(device_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(
            error = %e,
            device_id = device_id,
            "Failed to get location name by device id"
        );
        format!("Failed to get location name: {}", e)
    })?;

    Ok(row.map(|r| r.0))
}

/// Atualiza operation_status e last_seen_at do device por uuid.
/// Retorna número de linhas afetadas (0 se device não existir).
#[instrument(skip(pool), fields(device_uuid = %device_uuid))]
pub async fn update_device_operation_status_by_uuid_query(
    pool: &Pool<Sqlite>,
    device_uuid: &str,
    operation_status: &str,
    last_seen_at: &str,
) -> Result<u64, String> {
    let result = sqlx::query(
        r#"
        UPDATE devices
        SET operation_status = ?1, last_seen_at = ?2
        WHERE uuid = ?3 AND is_active = 1
        "#,
    )
    .bind(operation_status)
    .bind(last_seen_at)
    .bind(device_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, device_uuid = %device_uuid, "Failed to update device operation status");
        format!("Failed to update device: {}", e)
    })?;

    Ok(result.rows_affected())
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

// --- collector_notifications ---

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CollectorNotificationRow {
    pub uuid: String,
    pub notification_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub broker_uuid: Option<String>,
    pub device_uuid: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn insert_collector_notification(
    pool: &Pool<Sqlite>,
    user_id: i64,
    notification_type: &str,
    severity: &str,
    title: &str,
    message: &str,
    broker_uuid: Option<&str>,
    device_uuid: Option<&str>,
) -> Result<String, String> {
    let uuid = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO collector_notifications (uuid, user_id, notification_type, severity, title, message, broker_uuid, device_uuid)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
    )
    .bind(&uuid)
    .bind(user_id)
    .bind(notification_type)
    .bind(severity)
    .bind(title)
    .bind(message)
    .bind(broker_uuid)
    .bind(device_uuid)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to insert collector notification");
        format!("Failed to insert notification: {}", e)
    })?;
    Ok(uuid)
}

#[derive(Debug, Deserialize)]
pub struct CollectorNotificationListFilter {
    #[serde(default = "default_is_read", alias = "isRead")]
    pub is_read: String,
    #[serde(default = "default_severity")]
    pub severity: String,
}

impl Default for CollectorNotificationListFilter {
    fn default() -> Self {
        Self {
            is_read: default_is_read(),
            severity: default_severity(),
        }
    }
}

fn default_is_read() -> String {
    "no_read".to_string()
}
fn default_severity() -> String {
    "All".to_string()
}

#[derive(Debug, Deserialize)]
pub struct CollectorNotificationListParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size", alias = "pageSize")]
    pub page_size: u32,
    #[serde(default)]
    pub filter: CollectorNotificationListFilter,
}

fn default_page() -> u32 {
    1
}
fn default_page_size() -> u32 {
    20
}

#[derive(Debug, Serialize)]
pub struct CollectorNotificationListResponse {
    pub items: Vec<CollectorNotificationRow>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn list_collector_notifications_by_user(
    pool: &Pool<Sqlite>,
    user_id: i64,
    params: &CollectorNotificationListParams,
) -> Result<CollectorNotificationListResponse, String> {
    let page = params.page.max(1);
    let page_size = params.page_size.min(100).max(1);
    let offset = (page - 1) * page_size;

    let (is_read_filter, severity_filter) = (&params.filter.is_read, &params.filter.severity);

    let base_where = "user_id = ?";
    let mut where_clauses = vec![base_where.to_string()];

    let is_read_condition = match is_read_filter.as_str() {
        "no_read" => Some("is_read = 0"),
        "is_read" => Some("is_read = 1"),
        _ => None,
    };
    if let Some(c) = is_read_condition {
        where_clauses.push(c.to_string());
    }

    let severity_condition = if severity_filter != "All" {
        Some("severity = ?")
    } else {
        None
    };
    if severity_condition.is_some() {
        where_clauses.push(severity_condition.unwrap().to_string());
    }

    let where_sql = where_clauses.join(" AND ");
    tracing::info!(where_sql = %where_sql, user_id = user_id, "list_collector_notifications_by_user: executing");

    let count_sql = format!(
        "SELECT COUNT(*) FROM collector_notifications WHERE {}",
        where_sql
    );
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(user_id);
    if severity_filter != "All" {
        count_query = count_query.bind(severity_filter);
    }
    let total: i64 = count_query
        .fetch_one(pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to count collector notifications");
            format!("Failed to count notifications: {}", e)
        })?;

    let select_sql = format!(
        r#"
        SELECT uuid, notification_type, severity, title, message, broker_uuid, device_uuid, is_read, created_at
        FROM collector_notifications
        WHERE {}
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
        where_sql
    );
    let mut select_query = sqlx::query_as::<_, CollectorNotificationRow>(&select_sql)
        .bind(user_id);
    if severity_filter != "All" {
        select_query = select_query.bind(severity_filter);
    }
    let rows = select_query
        .bind(page_size as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            error!(error = %e, user_id = user_id, "Failed to list collector notifications");
            format!("Failed to list notifications: {}", e)
        })?;

    Ok(CollectorNotificationListResponse {
        items: rows,
        total,
        page,
        page_size,
    })
}

#[instrument(skip(pool), fields(uuid = %uuid, user_id = user_id))]
pub async fn get_collector_notification_by_uuid(
    pool: &Pool<Sqlite>,
    uuid: &str,
    user_id: i64,
) -> Result<Option<CollectorNotificationRow>, String> {
    let row = sqlx::query_as::<_, CollectorNotificationRow>(
        r#"
        SELECT uuid, notification_type, severity, title, message, broker_uuid, device_uuid, is_read, created_at
        FROM collector_notifications
        WHERE uuid = ?1 AND user_id = ?2
        LIMIT 1
        "#,
    )
    .bind(uuid)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        error!(error = %e, uuid = uuid, "Failed to get collector notification");
        format!("Failed to get notification: {}", e)
    })?;
    Ok(row)
}

#[instrument(skip(pool), fields(uuid = %uuid, user_id = user_id))]
pub async fn mark_collector_notification_read_by_uuid(
    pool: &Pool<Sqlite>,
    uuid: &str,
    user_id: i64,
) -> Result<(), String> {
    let result = sqlx::query(
        r#"
        UPDATE collector_notifications SET is_read = 1
        WHERE uuid = ?1 AND user_id = ?2
        "#,
    )
    .bind(uuid)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to mark notification as read");
        format!("Failed to mark notification: {}", e)
    })?;
    if result.rows_affected() == 0 {
        return Err("Notification not found".to_string());
    }
    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn mark_all_collector_notifications_read(
    pool: &Pool<Sqlite>,
    user_id: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE collector_notifications SET is_read = 1
        WHERE user_id = ?1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| {
        error!(error = %e, "Failed to mark all notifications as read");
        format!("Failed to mark notifications: {}", e)
    })?;
    Ok(())
}

#[instrument(skip(pool), fields(user_id = user_id))]
pub async fn count_unread_collector_notifications(
    pool: &Pool<Sqlite>,
    user_id: i64,
) -> Result<i64, String> {
    let result: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM collector_notifications
        WHERE user_id = ?1 AND is_read = 0
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        error!(error = %e, user_id = user_id, "Failed to count unread notifications");
        format!("Failed to count notifications: {}", e)
    })?;
    Ok(result.0)
}

// --- end collector_notifications ---

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


