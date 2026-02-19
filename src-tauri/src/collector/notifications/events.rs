use serde::{Deserialize, Serialize};
use chrono::Utc;

/// Types of notification events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    MqttConnectionLost,
    MqttConnectionRestored,
    DeviceOffline,
    CriticalError,
}

/// Notification event sent from collector to Tauri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEvent {
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub broker_uuid: Option<String>,
    pub device_uuid: Option<String>,
    pub user_id: Option<i64>,
    pub timestamp: String,
}

impl NotificationEvent {
    /// Creates a timestamp in ISO 8601 format (UTC)
    fn now_timestamp() -> String {
        Utc::now().to_rfc3339()
    }

    pub fn mqtt_connection_lost(broker_name: &str, broker_uuid: Option<String>, user_id: Option<i64>) -> Self {
        Self {
            notification_type: NotificationType::MqttConnectionLost,
            title: "MQTT Connection Lost".to_string(),
            message: format!("Connection to broker '{}' was lost", broker_name),
            broker_uuid,
            device_uuid: None,
            user_id,
            timestamp: Self::now_timestamp(),
        }
    }

    pub fn mqtt_connection_restored(broker_name: &str, broker_uuid: Option<String>, user_id: Option<i64>) -> Self {
        Self {
            notification_type: NotificationType::MqttConnectionRestored,
            title: "MQTT Connection Restored".to_string(),
            message: format!("Connection to broker '{}' was restored", broker_name),
            broker_uuid,
            device_uuid: None,
            user_id,
            timestamp: Self::now_timestamp(),
        }
    }

    pub fn critical_error(message: String, user_id: Option<i64>) -> Self {
        Self {
            notification_type: NotificationType::CriticalError,
            title: "Collector Error".to_string(),
            message: if message.len() > 200 {
                format!("{}...", &message[..200])
            } else {
                message
            },
            broker_uuid: None,
            device_uuid: None,
            user_id,
            timestamp: Self::now_timestamp(),
        }
    }
}

