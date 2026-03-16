use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::collector::model::MqttBroker;

/// Commands sent to the collector to manage user sessions
#[derive(Debug, Clone)]
pub enum CollectorCommand {
    /// User logged in - start monitoring their default broker
    UserLoggedIn { user_id: i64 },
    /// User logged out - stop monitoring
    UserLoggedOut,
    /// Connect to a specific broker (disconnects current if any). Broker becomes default.
    ConnectBroker { broker_uuid: String },
    /// Disconnect current broker, keep user logged in
    DisconnectBroker,
    /// Publish message to MQTT (e.g. trigger executor sending command to device). Only works when connected.
    PublishMessage { topic: String, payload: String },
}

/// Request to publish from monitor task (topic, payload).
pub type PublishRequest = (String, String);

/// Shared state for the collector service
/// Manages the current user session and broker connection
pub struct CollectorState {
    /// Current logged-in user ID (None = no user logged in)
    pub current_user_id: Arc<Mutex<Option<i64>>>,
    /// Current broker being monitored (None = not monitoring)
    pub current_broker: Arc<Mutex<Option<MqttBroker>>>,
    /// Channel sender for sending commands to the collector
    pub command_tx: mpsc::Sender<CollectorCommand>,
    /// Channel sender for publish requests (used by monitor task). Set when monitor starts, cleared when it stops.
    publish_tx: Arc<Mutex<Option<mpsc::Sender<PublishRequest>>>>,
}

impl CollectorState {
    /// Creates a new CollectorState with a command channel
    /// Returns the state and the receiver for processing commands
    pub fn new() -> (Self, mpsc::Receiver<CollectorCommand>) {
        let (tx, rx) = mpsc::channel(100);

        let state = Self {
            current_user_id: Arc::new(Mutex::new(None)),
            current_broker: Arc::new(Mutex::new(None)),
            command_tx: tx,
            publish_tx: Arc::new(Mutex::new(None)),
        };

        (state, rx)
    }

    /// Set the publish channel sender (called when monitor starts/stops).
    pub fn set_publish_tx(&self, tx: Option<mpsc::Sender<PublishRequest>>) {
        let mut guard = self.publish_tx.lock().unwrap();
        *guard = tx;
    }

    /// Send a publish request to the monitor. Fails if not connected or channel full.
    pub async fn send_publish(&self, topic: String, payload: String) -> Result<(), String> {
        let tx = {
            let guard = self.publish_tx.lock().unwrap();
            guard.clone()
        };
        match tx {
            Some(sender) => sender.send((topic, payload)).await.map_err(|e| {
                error!(error = %e, "send_publish: channel closed or full");
                format!("MQTT not connected or publish channel full: {}", e)
            }),
            None => Err("MQTT not connected".to_string()),
        }
    }
    
    /// Gets the current user ID
    pub fn get_current_user_id(&self) -> Option<i64> {
        *self.current_user_id.lock().unwrap()
    }
    
    /// Sets the current user ID
    pub fn set_current_user_id(&self, user_id: Option<i64>) {
        let mut current = self.current_user_id.lock().unwrap();
        *current = user_id;
        
        if let Some(id) = user_id {
            info!(user_id = id, "Collector state: user logged in");
        } else {
            info!("Collector state: user logged out");
        }
    }
    
    /// Gets the current broker being monitored
    pub fn get_current_broker(&self) -> Option<MqttBroker> {
        self.current_broker.lock().unwrap().clone()
    }
    
    /// Sets the current broker being monitored
    pub fn set_current_broker(&self, broker: Option<MqttBroker>) {
        let mut current = self.current_broker.lock().unwrap();
        *current = broker.clone();
        
        if let Some(b) = &broker {
            info!(broker_uuid = %b.uuid, broker_name = %b.name, "Collector state: broker set");
        } else {
            info!("Collector state: broker cleared");
        }
    }
    
    /// Sends a command to the collector
    /// Returns error if channel is closed
    pub async fn send_command(&self, command: CollectorCommand) -> Result<(), String> {
        self.command_tx
            .send(command)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send collector command");
                format!("Collector command channel closed: {}", e)
            })
    }
}

impl Clone for CollectorState {
    fn clone(&self) -> Self {
        Self {
            current_user_id: Arc::clone(&self.current_user_id),
            current_broker: Arc::clone(&self.current_broker),
            command_tx: self.command_tx.clone(),
            publish_tx: Arc::clone(&self.publish_tx),
        }
    }
}

