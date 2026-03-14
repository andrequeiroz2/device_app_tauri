use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tracing::{error, info, instrument, warn};
use tokio::time::sleep;
use crossbeam_channel::RecvTimeoutError;

use crate::collector::mqtt::client::MqttClient;
use crate::collector::persistence::query::{
    get_broker_by_uuid_and_user_query,
    get_default_broker_by_user_query,
    set_broker_as_default_query,
};
use crate::collector::notifications::sender::NotificationSender;
use crate::collector::notifications::events::NotificationEvent;
use crate::collector::state::{CollectorCommand, CollectorState};
use sqlx::Pool;
use sqlx::Sqlite;

/// Result of start_collector
pub struct StartCollectorResult {
    /// Shared state for sending commands to the collector
    pub state: CollectorState,
    /// Receiver for notification events (for tray listener)
    pub notification_rx: std::sync::mpsc::Receiver<crate::collector::notifications::events::NotificationEvent>,
}

/// Starts the collector in background.
/// Collector starts in idle state (no broker). Use CollectorState::send_command
/// to send UserLoggedIn/UserLoggedOut and trigger dynamic broker connect/disconnect.
///
/// Returns CollectorState and notification receiver for the Tauri app.
#[instrument(skip_all)]
pub async fn start_collector(
    pool: Pool<Sqlite>,
    app: AppHandle,
) -> Result<StartCollectorResult, Box<dyn std::error::Error>> {
    info!("Starting MQTT collector...");

    let (collector_state, command_rx) = CollectorState::new();
    let (notification_sender, notification_rx) =
        NotificationSender::new(collector_state.current_user_id.clone());

    // Start HTTP API server in background (logs localhost:port)
    crate::collector::api::main::start_api_server_background(pool.clone());

    // Spawn command loop
    tauri::async_runtime::spawn(collector_command_loop(
        command_rx,
        pool,
        Some(notification_sender),
        collector_state.clone(),
        app,
    ));

    info!("Collector started in idle state. Awaiting UserLoggedIn commands.");
    Ok(StartCollectorResult {
        state: collector_state,
        notification_rx,
    })
}

/// Main command loop. Processes UserLoggedIn/UserLoggedOut and orchestrates
/// broker connect/disconnect.
#[instrument(skip_all, name = "collector_command_loop")]
async fn collector_command_loop(
    mut command_rx: tokio::sync::mpsc::Receiver<CollectorCommand>,
    pool: Pool<Sqlite>,
    notification_sender: Option<NotificationSender>,
    collector_state: CollectorState,
    app: AppHandle,
) {
    let mut monitor_handle: Option<tauri::async_runtime::JoinHandle<()>> = None;
    let stop_requested = Arc::new(AtomicBool::new(false));

    while let Some(cmd) = command_rx.recv().await {
        match cmd {
            CollectorCommand::UserLoggedIn { user_id } => {
                info!(user_id = user_id, "Command: UserLoggedIn");

                // Stop current monitor if running
                stop_requested.store(true, Ordering::SeqCst);
                if let Some(handle) = monitor_handle.take() {
                    let _ = handle.await;
                }
                stop_requested.store(false, Ordering::SeqCst);

                collector_state.set_current_user_id(Some(user_id));

                // Fetch broker for user
                let broker = match get_default_broker_by_user_query(&pool, user_id).await {
                    Ok(Some(b)) => {
                        info!(broker_uuid = %b.uuid, broker_name = %b.name, "Default broker found for user");
                        b
                    }
                    Ok(None) => {
                        warn!(user_id = user_id, "No default broker for user");
                        collector_state.set_current_broker(None);
                        continue;
                    }
                    Err(e) => {
                        error!(user_id = user_id, error = %e, "Failed to get default broker");
                        if let Some(ref sender) = notification_sender {
                            sender.send(NotificationEvent::critical_error(
                                format!("Failed to get broker: {}", e),
                                Some(user_id),
                            ));
                        }
                        continue;
                    }
                };

                collector_state.set_current_broker(Some(broker.clone()));

                let pool_clone = pool.clone();
                let sender_clone = notification_sender.clone();
                let stop = Arc::clone(&stop_requested);
                let state_clone = collector_state.clone();
                let app_clone = app.clone();

                let handle = tauri::async_runtime::spawn(async move {
                    run_monitor_with_reconnect(broker, pool_clone, sender_clone, stop, state_clone, app_clone)
                        .await;
                });
                monitor_handle = Some(handle);
            }
            CollectorCommand::UserLoggedOut => {
                info!("Command: UserLoggedOut");

                stop_requested.store(true, Ordering::SeqCst);
                if let Some(handle) = monitor_handle.take() {
                    let _ = handle.await;
                }
                stop_requested.store(false, Ordering::SeqCst);

                collector_state.set_current_user_id(None);
                collector_state.set_current_broker(None);
                info!("Collector stopped. Idle.");
            }
            CollectorCommand::ConnectBroker { broker_uuid } => {
                info!(broker_uuid = %broker_uuid, "Command: ConnectBroker");

                let user_id = match collector_state.get_current_user_id() {
                    Some(id) => id,
                    None => {
                        warn!("ConnectBroker ignored: no user logged in");
                        continue;
                    }
                };

                stop_requested.store(true, Ordering::SeqCst);
                if let Some(handle) = monitor_handle.take() {
                    let _ = handle.await;
                }
                stop_requested.store(false, Ordering::SeqCst);

                if let Err(e) = set_broker_as_default_query(&pool, user_id, &broker_uuid).await {
                    error!(error = %e, "Failed to set broker as default");
                    if let Some(ref sender) = notification_sender {
                        sender.send(NotificationEvent::critical_error(
                            format!("Failed to set broker default: {}", e),
                            Some(user_id),
                        ));
                    }
                    continue;
                }

                let broker = match get_broker_by_uuid_and_user_query(&pool, &broker_uuid, user_id).await
                {
                    Ok(Some(b)) => {
                        info!(broker_uuid = %b.uuid, broker_name = %b.name, "Broker found for ConnectBroker");
                        b
                    }
                    Ok(None) => {
                        warn!(broker_uuid = %broker_uuid, "Broker not found or inactive");
                        if let Some(ref sender) = notification_sender {
                            sender.send(NotificationEvent::critical_error(
                                "Broker not found or inactive".to_string(),
                                Some(user_id),
                            ));
                        }
                        continue;
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to get broker");
                        if let Some(ref sender) = notification_sender {
                            sender.send(NotificationEvent::critical_error(
                                format!("Failed to get broker: {}", e),
                                Some(user_id),
                            ));
                        }
                        continue;
                    }
                };

                collector_state.set_current_broker(Some(broker.clone()));

                let pool_clone = pool.clone();
                let sender_clone = notification_sender.clone();
                let stop = Arc::clone(&stop_requested);
                let state_clone = collector_state.clone();
                let app_clone = app.clone();

                let handle = tauri::async_runtime::spawn(async move {
                    run_monitor_with_reconnect(broker, pool_clone, sender_clone, stop, state_clone, app_clone)
                        .await;
                });
                monitor_handle = Some(handle);
            }
            CollectorCommand::DisconnectBroker => {
                info!("Command: DisconnectBroker");

                stop_requested.store(true, Ordering::SeqCst);
                if let Some(handle) = monitor_handle.take() {
                    let _ = handle.await;
                }
                stop_requested.store(false, Ordering::SeqCst);

                collector_state.set_current_broker(None);
                info!("Broker disconnected. User remains logged in.");
            }
        }
    }

    info!("Collector command channel closed, shutting down");
}

/// Runs connect + monitor loop with reconnection. Stops when stop_requested is set.
async fn run_monitor_with_reconnect(
    broker: crate::collector::model::MqttBroker,
    pool: Pool<Sqlite>,
    notification_sender: Option<NotificationSender>,
    stop_requested: Arc<AtomicBool>,
    _collector_state: CollectorState,
    app: AppHandle,
) {
    let mut backoff_secs = 1u64;
    let mut is_reconnect = false;

    loop {
        if stop_requested.load(Ordering::SeqCst) {
            break;
        }
        match try_connect_and_monitor(&broker, &pool, notification_sender.as_ref(), &stop_requested, is_reconnect, &app).await {
            Ok(true) => {
                // Stopped by command (UserLoggedOut or broker switch)
                break;
            }
            Ok(false) => {
                warn!("MQTT connection closed unexpectedly. Reconnecting...");
                is_reconnect = true;
                if let Some(ref sender) = notification_sender {
                    sender.send(NotificationEvent::mqtt_connection_lost(
                        &broker.name,
                        Some(broker.uuid.clone()),
                        Some(broker.user_id),
                    ));
                }
            }
            Err(e) => {
                error!(error = %e, "MQTT connection error. Reconnecting...");
                // Only notify on first connection failure; if we already sent MqttConnectionLost, skip to avoid noise
                if !is_reconnect {
                    if let Some(ref sender) = notification_sender {
                        sender.send(NotificationEvent::critical_error(
                            format!("MQTT connection error: {}", e),
                            Some(broker.user_id),
                        ));
                    }
                }
                is_reconnect = true;
            }
        }

        if stop_requested.load(Ordering::SeqCst) {
            break;
        }

        let delay = Duration::from_secs(backoff_secs.min(30));
        sleep(delay).await;
        backoff_secs = (backoff_secs * 2).min(30);
    }
}

/// Connects to broker, subscribes, monitors messages.
/// Returns Ok(true) if stopped by command, Ok(false) if connection lost, Err on connection error.
/// When is_reconnect is true, sends MqttConnectionRestored on success (only when we recover from a previous loss).
async fn try_connect_and_monitor(
    broker: &crate::collector::model::MqttBroker,
    pool: &Pool<Sqlite>,
    notification_sender: Option<&NotificationSender>,
    stop_requested: &AtomicBool,
    is_reconnect: bool,
    app: &AppHandle,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut client = MqttClient::new(broker.clone())?;

    client.connect().await?;

    if is_reconnect {
        if let Some(sender) = notification_sender {
            sender.send(NotificationEvent::mqtt_connection_restored(
                &broker.name,
                Some(broker.uuid.clone()),
                Some(broker.user_id),
            ));
        }
    }

    client.subscribe(&format!("{}/+/status", broker.uuid), 0).await?;
    client.subscribe(&format!("{}/+/data", broker.uuid), 0).await?;

    let rx = client.start_consuming();
    info!("MQTT monitoring started. Waiting for messages...");

    loop {
        if stop_requested.load(Ordering::SeqCst) {
            client.disconnect().await?;
            return Ok(true);
        }

        let result = rx.recv_timeout(Duration::from_millis(500));

        match result {
            Ok(Some(message)) => {
                crate::collector::mqtt::handler::handle_mqtt_message(message, pool, app).await;
            }
            Ok(None) => {
                warn!("Message stream closed");
                break;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                warn!("Message channel disconnected");
                break;
            }
        }

        if !client.is_connected() {
            warn!("MQTT connection lost");
            break;
        }
    }

    // Connection was lost (stream closed). disconnect() may fail with "Client disconnected" - ignore it.
    let _ = client.disconnect().await;
    Ok(false)
}
