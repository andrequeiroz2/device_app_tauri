use paho_mqtt::{Client, ConnectOptionsBuilder, CreateOptionsBuilder, Message};
use std::time::Duration;
use tracing::{error, info};
use crate::collector::model::MqttBroker;

pub struct MqttClient {
    client: Client,
    broker: MqttBroker,
}

impl MqttClient {
    pub fn new(broker: MqttBroker) -> Result<Self, String> {
        let server_uri = format!("{}://{}:{}", 
            if broker.use_tls { "ssl" } else { "tcp" },
            broker.host,
            broker.port
        );

        info!(
            server_uri = %server_uri,
            broker_name = %broker.name,
            "Creating MQTT client"
        );

        let create_opts = CreateOptionsBuilder::new()
            .server_uri(&server_uri)
            .client_id(broker.client_id.as_deref().unwrap_or("collector"))
            .finalize();

        let client = Client::new(create_opts)
            .map_err(|e| format!("Failed to create MQTT client: {}", e))?;

        Ok(MqttClient {
            client,
            broker,
        })
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        let mut conn_opts = ConnectOptionsBuilder::new();

        // Authentication
        if let (Some(username), Some(password)) = (&self.broker.username, &self.broker.password) {
            conn_opts.user_name(username);
            conn_opts.password(password);
        }

        // Keep alive
        conn_opts.keep_alive_interval(Duration::from_secs(self.broker.keep_alive_interval as u64));
        
        // Clean session
        conn_opts.clean_session(self.broker.clean_session);

        // Connection timeout
        conn_opts.connect_timeout(Duration::from_secs(self.broker.connection_timeout_secs as u64));

        // Last Will and Testament
        if let (Some(topic), Some(message)) = (&self.broker.last_will_topic, &self.broker.last_will_message) {
            let will = Message::new(topic, message.as_bytes(), self.broker.last_will_qos as i32);
            conn_opts.will_message(will);
        }

        let conn_opts = conn_opts.finalize();

        info!(
            broker_name = %self.broker.name,
            host = %self.broker.host,
            port = self.broker.port,
            "Connecting to MQTT broker"
        );

        self.client.connect(conn_opts)
            .map_err(|e| {
                error!(error = %e, "Failed to connect to MQTT broker");
                format!("Failed to connect to MQTT broker: {}", e)
            })?;

        info!(
            broker_name = %self.broker.name,
            "Connected to MQTT broker successfully"
        );

        Ok(())
    }

    pub async fn subscribe(&self, topic: &str, qos: i32) -> Result<(), String> {
        info!(topic = %topic, qos = qos, "Subscribing to MQTT topic");
        
        self.client.subscribe(topic, qos)
            .map_err(|e| {
                error!(error = %e, topic = %topic, "Failed to subscribe to topic");
                format!("Failed to subscribe to topic {}: {}", topic, e)
            })?;

        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), String> {
        info!("Disconnecting from MQTT broker");
        self.client.disconnect(None)
            .map_err(|e| format!("Failed to disconnect: {}", e))?;
        Ok(())
    }

    pub fn start_consuming(&self) -> paho_mqtt::Receiver<Option<Message>> {
        self.client.start_consuming()
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_connected()
    }
}


