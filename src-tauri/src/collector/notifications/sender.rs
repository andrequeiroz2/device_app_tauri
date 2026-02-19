use tokio::sync::mpsc;
use tracing::{error, warn};
use crate::collector::notifications::events::NotificationEvent;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Rate limiter for notifications
/// Prevents spam by limiting notifications per type
struct NotificationRateLimiter {
    last_sent: HashMap<String, Instant>,
    min_interval: Duration,
}

impl NotificationRateLimiter {
    fn new(min_interval_secs: u64) -> Self {
        Self {
            last_sent: HashMap::new(),
            min_interval: Duration::from_secs(min_interval_secs),
        }
    }

    fn should_send(&mut self, key: &str) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_sent.get(key) {
            if now.duration_since(*last) < self.min_interval {
                return false;
            }
        }
        self.last_sent.insert(key.to_string(), now);
        true
    }
}

/// Notification sender with rate limiting
pub struct NotificationSender {
    tx: mpsc::Sender<NotificationEvent>,
    rate_limiter: std::sync::Mutex<NotificationRateLimiter>,
}

impl NotificationSender {
    pub fn new() -> (Self, mpsc::Receiver<NotificationEvent>) {
        let (tx, rx) = mpsc::channel(100);
        let sender = Self {
            tx,
            rate_limiter: std::sync::Mutex::new(NotificationRateLimiter::new(60)), // 1 minuto
        };
        (sender, rx)
    }

    /// Sends a notification event (with rate limiting)
    pub fn send(&self, event: NotificationEvent) {
        let key = match &event.notification_type {
            crate::collector::notifications::events::NotificationType::MqttConnectionLost => {
                format!("mqtt_lost_{}", event.broker_uuid.as_deref().unwrap_or("unknown"))
            }
            crate::collector::notifications::events::NotificationType::MqttConnectionRestored => {
                format!("mqtt_restored_{}", event.broker_uuid.as_deref().unwrap_or("unknown"))
            }
            crate::collector::notifications::events::NotificationType::DeviceOffline => {
                format!("device_offline_{}", event.device_uuid.as_deref().unwrap_or("unknown"))
            }
            crate::collector::notifications::events::NotificationType::CriticalError => {
                "critical_error".to_string()
            }
        };

        let should_send = {
            let mut limiter = self.rate_limiter.lock().unwrap();
            limiter.should_send(&key)
        };

        if !should_send {
            warn!(key = %key, "Notification rate limited, skipping");
            return;
        }

        if let Err(e) = self.tx.try_send(event) {
            error!(error = %e, "Failed to send notification event (channel full or closed)");
        }
    }
}

impl Clone for NotificationSender {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rate_limiter: std::sync::Mutex::new(NotificationRateLimiter::new(60)),
        }
    }
}

