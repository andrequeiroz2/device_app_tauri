use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::error;
use crate::collector::notifications::events::NotificationEvent;

/// Notification sender with user-based filtering
pub struct NotificationSender {
    tx: mpsc::Sender<NotificationEvent>,
    /// Shared reference to current logged-in user. Only events matching this user are sent.
    current_user_id: Arc<Mutex<Option<i64>>>,
}

impl NotificationSender {
    /// Creates a new NotificationSender that filters events by current_user_id.
    /// Only events where event.user_id == current_user_id are forwarded.
    /// When current_user_id is None (logged out), no events are sent.
    pub fn new(current_user_id: Arc<Mutex<Option<i64>>>) -> (Self, mpsc::Receiver<NotificationEvent>) {
        let (tx, rx) = mpsc::channel(100);
        let sender = Self {
            tx,
            current_user_id,
        };
        (sender, rx)
    }

    /// Sends a notification event (with user filter)
    pub fn send(&self, event: NotificationEvent) {
        let current_id = *self.current_user_id.lock().unwrap();
        if let Some(id) = current_id {
            if event.user_id != Some(id) {
                return;
            }
        } else {
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
            current_user_id: Arc::clone(&self.current_user_id),
        }
    }
}
