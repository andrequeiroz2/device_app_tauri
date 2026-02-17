use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::instrument;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct MqttBroker {
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
    pub last_connected_at: Option<String>,
    pub last_connection_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MqttBrokerPublic {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
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
    pub last_connected_at: Option<String>,
    pub last_connection_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MqttBrokerCreateInput {
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: Option<i32>, // Default: 1883
    pub username: Option<String>,
    pub password: Option<String>, // Será criptografado
    pub use_tls: Option<bool>, // Default: false
    pub ca_certificate_path: Option<String>,
    pub client_certificate_path: Option<String>,
    pub client_key_path: Option<String>,
    pub insecure_tls: Option<bool>, // Default: false
    pub client_id: Option<String>,
    pub keep_alive_interval: Option<i32>, // Default: 60
    pub clean_session: Option<bool>, // Default: true
    pub connection_timeout_secs: Option<i32>, // Default: 30
    pub operation_timeout_secs: Option<i32>, // Default: 30
    pub last_will_topic: Option<String>,
    pub last_will_message: Option<String>,
    pub last_will_qos: Option<i32>, // Default: 0
    pub last_will_retain: Option<bool>, // Default: false
    pub is_default: Option<bool>, // Default: false
}

#[derive(Debug)]
pub struct MqttBrokerCreateDB {
    pub uuid: String,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub password: Option<String>, // Já criptografado
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
    pub is_default: bool,
}

impl MqttBrokerCreateInput {
    /// Validate mandatory fields and sanitize for insertion.
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), String> {
        let name = self.name.trim();
        let host = self.host.trim();

        // Validate name
        if name.is_empty() {
            return Err("Name is required".to_string());
        }
        if name.len() > 255 {
            return Err("Name is too long (max 255)".to_string());
        }

        // Validate host
        if host.is_empty() {
            return Err("Host is required".to_string());
        }
        if host.len() > 255 {
            return Err("Host is too long (max 255)".to_string());
        }

        // Validate port if provided
        if let Some(port) = self.port {
            if port <= 0 || port > 65535 {
                return Err("Port must be between 1 and 65535".to_string());
            }
        }

        // Validate username if provided
        if let Some(ref username) = self.username {
            let username = username.trim();
            if username.is_empty() {
                return Err("Username cannot be empty if provided".to_string());
            }
            if username.len() > 255 {
                return Err("Username is too long (max 255)".to_string());
            }
        }

        // Validate password if provided
        if let Some(ref password) = self.password {
            if password.is_empty() {
                return Err("Password cannot be empty if provided".to_string());
            }
        }

        // Validate keep_alive_interval if provided
        if let Some(keep_alive) = self.keep_alive_interval {
            if keep_alive <= 0 {
                return Err("Keep alive interval must be greater than 0".to_string());
            }
            if keep_alive > 65535 {
                return Err("Keep alive interval is too large (max 65535)".to_string());
            }
        }

        // Validate connection_timeout_secs if provided
        if let Some(timeout) = self.connection_timeout_secs {
            if timeout <= 0 {
                return Err("Connection timeout must be greater than 0".to_string());
            }
            if timeout > 300 {
                return Err("Connection timeout is too large (max 300)".to_string());
            }
        }

        // Validate operation_timeout_secs if provided
        if let Some(timeout) = self.operation_timeout_secs {
            if timeout <= 0 {
                return Err("Operation timeout must be greater than 0".to_string());
            }
            if timeout > 300 {
                return Err("Operation timeout is too large (max 300)".to_string());
            }
        }

        // Validate last_will_qos if provided
        if let Some(qos) = self.last_will_qos {
            if !matches!(qos, 0 | 1 | 2) {
                return Err("Last will QoS must be 0, 1, or 2".to_string());
            }
        }

        Ok(())
    }

    /// Build the DB struct with defaults applied.
    pub fn to_db(&self, user_id: i64) -> MqttBrokerCreateDB {
        let use_tls = self.use_tls.unwrap_or(false);
        let port = self.port.unwrap_or(if use_tls { 8883 } else { 1883 });

        MqttBrokerCreateDB {
            uuid: uuid::Uuid::new_v4().to_string(),
            user_id,
            name: self.name.trim().to_string(),
            description: self.description.as_ref().map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
            host: self.host.trim().to_string(),
            port,
            username: self.username.as_ref().map(|u| u.trim().to_string()).filter(|u| !u.is_empty()),
            password: None, // Será preenchido após criptografia no handler
            use_tls,
            ca_certificate_path: self.ca_certificate_path.as_ref().map(|p| p.trim().to_string()).filter(|p| !p.is_empty()),
            client_certificate_path: self.client_certificate_path.as_ref().map(|p| p.trim().to_string()).filter(|p| !p.is_empty()),
            client_key_path: self.client_key_path.as_ref().map(|p| p.trim().to_string()).filter(|p| !p.is_empty()),
            insecure_tls: self.insecure_tls.unwrap_or(false),
            client_id: self.client_id.as_ref().map(|c| c.trim().to_string()).filter(|c| !c.is_empty()),
            keep_alive_interval: self.keep_alive_interval.unwrap_or(60),
            clean_session: self.clean_session.unwrap_or(true),
            connection_timeout_secs: self.connection_timeout_secs.unwrap_or(30),
            operation_timeout_secs: self.operation_timeout_secs.unwrap_or(30),
            last_will_topic: self.last_will_topic.as_ref().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()),
            last_will_message: self.last_will_message.as_ref().map(|m| m.trim().to_string()).filter(|m| !m.is_empty()),
            last_will_qos: self.last_will_qos.unwrap_or(0),
            last_will_retain: self.last_will_retain.unwrap_or(false),
            is_default: self.is_default.unwrap_or(false),
        }
    }
}

impl From<MqttBroker> for MqttBrokerPublic {
    fn from(broker: MqttBroker) -> Self {
        MqttBrokerPublic {
            uuid: broker.uuid,
            name: broker.name,
            description: broker.description,
            host: broker.host,
            port: broker.port,
            username: broker.username,
            // password NÃO é exposto
            use_tls: broker.use_tls,
            ca_certificate_path: broker.ca_certificate_path,
            client_certificate_path: broker.client_certificate_path,
            client_key_path: broker.client_key_path,
            insecure_tls: broker.insecure_tls,
            client_id: broker.client_id,
            keep_alive_interval: broker.keep_alive_interval,
            clean_session: broker.clean_session,
            connection_timeout_secs: broker.connection_timeout_secs,
            operation_timeout_secs: broker.operation_timeout_secs,
            last_will_topic: broker.last_will_topic,
            last_will_message: broker.last_will_message,
            last_will_qos: broker.last_will_qos,
            last_will_retain: broker.last_will_retain,
            is_active: broker.is_active,
            is_connected: broker.is_connected,
            is_default: broker.is_default,
            last_connected_at: broker.last_connected_at,
            last_connection_error: broker.last_connection_error,
            created_at: broker.created_at,
            updated_at: broker.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MqttBrokerStatusFilter {
    /// Show only active brokers (is_active = true)
    Active,
    /// Show all brokers (active and inactive)
    All,
}

impl Default for MqttBrokerStatusFilter {
    fn default() -> Self {
        MqttBrokerStatusFilter::Active
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct MqttBrokerFilter {
    #[serde(default)]
    pub status: MqttBrokerStatusFilter,
    /// Filter by broker name (partial match, case-insensitive)
    pub name: Option<String>,
    /// Filter by port number
    pub port: Option<u16>,
    /// Show only default brokers (is_default = true)
    pub default: Option<bool>,
    /// Show only connected brokers (is_connected = true)
    pub connected: Option<bool>,
}

impl MqttBrokerFilter {
    /// Returns true if we should show all brokers (including inactive)
    pub fn show_all(&self) -> bool {
        matches!(self.status, MqttBrokerStatusFilter::All)
    }

    /// Returns true if any filter is set
    pub fn has_filters(&self) -> bool {
        self.name.is_some()
            || self.port.is_some()
            || self.default.is_some()
            || self.connected.is_some()
    }

    /// Returns true if should filter by default brokers only
    pub fn is_default_only(&self) -> bool {
        self.default == Some(true)
    }

    /// Returns true if should filter by connected brokers only
    pub fn is_connected_only(&self) -> bool {
        self.connected == Some(true)
    }
}

#[derive(Debug, Deserialize)]
pub struct MqttBrokerListParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    #[serde(default)]
    pub filter: MqttBrokerFilter,
}

#[derive(Debug, Serialize)]
pub struct MqttBrokerListResponse {
    pub items: Vec<MqttBrokerPublic>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct MqttBrokerDeleteInput {
    pub uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct MqttBrokerUpdateInput {
    pub uuid: String,
    pub is_active: Option<bool>,
}

impl MqttBrokerUpdateInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.uuid.trim().is_empty() {
            return Err("UUID cannot be empty".to_string());
        }
        Ok(())
    }
}

