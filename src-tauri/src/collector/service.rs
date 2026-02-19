use tracing::{info, error, warn};
use crate::collector::mqtt::client::MqttClient;
use crate::collector::persistence::query::get_default_broker_query;
use crate::collector::notifications::sender::NotificationSender;
use crate::collector::notifications::events::NotificationEvent;
use sqlx::Pool;
use sqlx::Sqlite;
use std::time::Duration;
use tokio::time::sleep;

/// Starts the collector in background
/// Receives the shared SQLite pool from the Tauri application
/// Returns the notification receiver channel for Tauri to consume
pub async fn start_collector(
    pool: Pool<Sqlite>,
    notification_sender: Option<NotificationSender>,
) -> Result<tokio::sync::mpsc::Receiver<NotificationEvent>, Box<dyn std::error::Error>> {
    info!("Starting MQTT collector...");

    // Note: mqtt_messages table is now created in api::database::schema_sqlite::init_sqlite_schema()
    // which is called during app initialization in get_sqlite_pool()
    // No need to initialize it here anymore.

    // Start HTTP API server in background
    crate::collector::api::main::start_api_server_background(pool.clone());

    info!("Searching for default broker...");

    // Get default broker
    let broker = match get_default_broker_query(&pool).await? {
        Some(broker) => {
            info!(broker_uuid = %broker.uuid, broker_name = %broker.name, "Default broker found");
            broker
        }
        None => {
            error!("No default broker found. Please configure a default broker in the application.");
            return Err("No default broker found".into());
        }
    };

    // Use provided notification sender
    let notification_sender_for_loop = notification_sender;
    
    // Start reconnection loop in background
    let broker_clone = broker.clone();
    let pool_clone = pool.clone();
    let sender_clone = notification_sender_for_loop.clone();
    
    tauri::async_runtime::spawn(async move {
        loop {
            match try_connect_and_monitor(&broker_clone, &pool_clone, sender_clone.as_ref()).await {
                Ok(_) => {
                    warn!("MQTT connection closed unexpectedly. Attempting to reconnect...");
                    if let Some(sender) = &sender_clone {
                        sender.send(NotificationEvent::mqtt_connection_lost(
                            &broker_clone.name,
                            Some(broker_clone.uuid.clone()),
                            Some(broker_clone.user_id),
                        ));
                    }
                }
                Err(e) => {
                    error!(error = %e, "MQTT connection error. Attempting to reconnect...");
                    if let Some(sender) = &sender_clone {
                        sender.send(NotificationEvent::critical_error(
                            format!("MQTT connection error: {}", e),
                            Some(broker_clone.user_id),
                        ));
                    }
                }
            }

            // Exponential backoff: 1s, 2s, 4s, 8s, max 30s
            sleep(Duration::from_secs(1)).await;
        }
    });

    // Return a dummy receiver since NotificationSender already has its own channel
    // The actual receiver is created in lib.rs when NotificationSender::new() is called
    let (_tx, rx) = tokio::sync::mpsc::channel(1);
    Ok(rx)
}

async fn try_connect_and_monitor(
    broker: &crate::collector::model::MqttBroker,
    pool: &Pool<Sqlite>,
    notification_sender: Option<&NotificationSender>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MqttClient::new(broker.clone())?;

    // Connect
    client.connect().await?;
    
    // Send connection restored notification
    if let Some(sender) = notification_sender {
        sender.send(NotificationEvent::mqtt_connection_restored(
            &broker.name,
            Some(broker.uuid.clone()),
            Some(broker.user_id),
        ));
    }

    // Subscribe to topics following pattern: {broker_uuid}/{device_uuid}/...
    // Using wildcards to subscribe to all devices for this broker
    // Pattern: {broker_uuid}/+/status and {broker_uuid}/+/data
    client.subscribe(&format!("{}/+/status", broker.uuid), 0).await?;
    client.subscribe(&format!("{}/+/data", broker.uuid), 0).await?;

    // Start consuming messages
    let rx = client.start_consuming();

    info!("MQTT monitoring started. Waiting for messages...");

    // Message receiving loop
    loop {
        match rx.recv() {
            Ok(Some(message)) => {
                crate::collector::mqtt::handler::handle_mqtt_message(message, pool).await;
            }
            Ok(None) => {
                warn!("Message stream closed");
                break;
            }
            Err(e) => {
                error!(error = %e, "Error receiving message");
                break;
            }
        }

        // Check if still connected
        if !client.is_connected() {
            warn!("MQTT connection lost");
            break;
        }
    }

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
