use sqlx::FromRow;
use serde::{Deserialize, Serialize};

#[derive(Debug, FromRow)]
pub struct MqttBrokerRow {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
    pub ca_certificate_path: Option<String>,
    pub client_certificate_path: Option<String>,
    pub client_key_path: Option<String>,
    pub insecure_tls: bool,
    pub client_id: Option<String>,
    pub keep_alive_interval: i32,
    pub clean_session: bool,
    pub connection_timeout_secs: i32,
    pub operation_timeout_secs: i32,
    pub last_will_topic: Option<String>,
    pub last_will_message: Option<String>,
    pub last_will_qos: i32,
    pub last_will_retain: bool,
    pub is_active: bool,
    pub is_connected: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct MqttBroker {
    pub uuid: String,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
    pub client_id: Option<String>,
    pub keep_alive_interval: i32,
    pub clean_session: bool,
    pub connection_timeout_secs: i32,
    pub last_will_topic: Option<String>,
    pub last_will_message: Option<String>,
    pub last_will_qos: i32,
    pub user_id: i64,
}

impl From<MqttBrokerRow> for MqttBroker {
    fn from(row: MqttBrokerRow) -> Self {
        MqttBroker {
            uuid: row.uuid,
            name: row.name,
            host: row.host,
            port: row.port,
            username: row.username,
            password: row.password,
            use_tls: row.use_tls,
            client_id: row.client_id,
            keep_alive_interval: row.keep_alive_interval,
            clean_session: row.clean_session,
            connection_timeout_secs: row.connection_timeout_secs,
            last_will_topic: row.last_will_topic,
            last_will_message: row.last_will_message,
            last_will_qos: row.last_will_qos,
            user_id: row.user_id,
        }
    }
}

// API Models
#[derive(Debug, Serialize)]
pub struct CollectorStatus {
    pub running: bool,
    pub mqtt_connected: bool,
    pub broker_name: Option<String>,
    pub last_message_at: Option<String>,
    pub total_messages: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MqttMessage {
    pub id: i64,
    pub topic: String,
    pub broker_uuid: Option<String>,
    pub device_uuid: Option<String>,
    pub payload: String,
    pub qos: i32,
    pub retain: bool,
    pub received_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PublishMessageInput {
    pub topic: String,
    pub payload: String,
    pub qos: Option<i32>,
    pub retain: Option<bool>,
}
