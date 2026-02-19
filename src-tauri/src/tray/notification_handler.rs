use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use tracing::{info, error};
use crate::collector::notifications::events::NotificationEvent;

/// Handles notification events from the collector
pub async fn handle_notification_event(app: &AppHandle, event: NotificationEvent) {
    info!(
        notification_type = ?event.notification_type,
        title = %event.title,
        message = %event.message,
        timestamp = %event.timestamp,
        user_id = ?event.user_id,
        broker_uuid = ?event.broker_uuid,
        "Handling notification event"
    );

    // Get notification instance from AppHandle
    let notification = app.notification();
    
    // Build and show notification using builder pattern
    match notification.builder()
        .title(&event.title)
        .body(&event.message)
        .show() {
        Ok(_) => {
            info!(title = %event.title, "Notification sent successfully");
        }
        Err(e) => {
            error!(error = %e, title = %event.title, "Failed to send notification");
        }
    }
}

/// Starts the notification listener loop
pub fn start_notification_listener(
    app: AppHandle,
    mut rx: tokio::sync::mpsc::Receiver<NotificationEvent>,
) {
    tauri::async_runtime::spawn(async move {
        info!("Notification listener started");
        
        while let Some(event) = rx.recv().await {
            handle_notification_event(&app, event).await;
        }
        
        info!("Notification listener stopped");
    });
}
