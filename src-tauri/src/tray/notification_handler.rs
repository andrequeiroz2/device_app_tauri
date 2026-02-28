use sqlx::Pool;
use sqlx::Sqlite;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tracing::{error, info, warn};

use crate::collector::notifications::events::NotificationEvent;
use crate::collector::persistence::query::insert_collector_notification;

/// Handles notification events from the collector: persists to DB, then shows toast.
pub async fn handle_notification_event(
    app: &AppHandle,
    pool: &Pool<Sqlite>,
    event: NotificationEvent,
) {
    info!(
        notification_type = ?event.notification_type,
        title = %event.title,
        "Handling notification event"
    );

    // Persist before toast (only if we have user_id)
    if let Some(user_id) = event.user_id {
        let notification_type_str = event.notification_type.as_str();
        let severity = event.notification_type.as_severity();
        match insert_collector_notification(
            pool,
            user_id,
            notification_type_str,
            severity,
            &event.title,
            &event.message,
            event.broker_uuid.as_deref(),
            event.device_uuid.as_deref(),
        )
        .await
        {
            Ok(_) => {
                let _ = app.emit("collector-notification-added", ());
            }
            Err(e) => {
                error!(error = %e, "Failed to persist collector notification");
            }
        }
    }

    // Show toast in background
    let app = app.clone();
    let title = event.title.clone();
    let message = event.message.clone();
    tauri::async_runtime::spawn(async move {
        let notification = app.notification();
        if let Err(e) = notification.builder().title(&title).body(&message).show() {
            error!(error = %e, title = %title, "Failed to send notification");
        }
    });
}

/// Starts the notification listener loop.
/// Persists each event to collector_notifications before showing the toast.
pub fn start_notification_listener(
    app: AppHandle,
    rx: std::sync::mpsc::Receiver<NotificationEvent>,
    pool: Pool<Sqlite>,
) {
    std::thread::spawn(move || {
        info!("Notification listener started");

        while let Ok(event) = rx.recv() {
            tauri::async_runtime::block_on(handle_notification_event(&app, &pool, event));
        }

        warn!("Notification listener stopped");
    });
}
