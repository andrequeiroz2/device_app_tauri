use serde::{Deserialize, Serialize};
use tracing::{debug, instrument, warn};

use super::error::ProvisioningError;
use super::serial::SerialConnection;

/// Command sent to device
#[derive(Debug, Serialize)]
struct Command {
    cmd: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

/// Generic response from device
#[derive(Debug, Deserialize)]
struct GenericResponse {
    ok: Option<bool>,
    error: Option<String>,
}

/// Response from ping command
#[derive(Debug, Deserialize)]
pub struct PingResponse {
    pub ok: bool,
    pub version: Option<String>,
}

/// Device information returned by get_info command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub ok: Option<bool>,
    pub adopted_status: i32,
    pub device_type: String,
    #[serde(default)]
    pub sensor_type: Option<String>,
    #[serde(default)]
    pub actuator_type: Option<String>,
    pub boarder_type: String,
    pub mac_address: String,
    #[serde(default)]
    pub device_scale: Option<serde_json::Value>,
    /// Sensor: { measurement: { unit, min_reading, max_reading } }
    #[serde(default)]
    pub parameter_ranges: Option<serde_json::Value>,
    /// Actuator: { type: "discrete"|"range", ... }
    #[serde(default)]
    pub command_spec: Option<serde_json::Value>,
    #[serde(default)]
    pub firmware_version: Option<String>,
    /// Present when adopted_status=1; used to check if device belongs to logged user
    #[serde(default)]
    pub user_uuid: Option<String>,
}

impl DeviceInfo {
    /// Check if device is already adopted
    pub fn is_adopted(&self) -> bool {
        self.adopted_status == 1
    }

    /// Get normalized device type (lowercase)
    pub fn normalized_device_type(&self) -> String {
        self.device_type.to_lowercase()
    }

    /// Validate device info
    pub fn validate(&self) -> Result<(), ProvisioningError> {
        if self.mac_address.is_empty() {
            return Err(ProvisioningError::InvalidMacAddress("empty".to_string()));
        }

        if !Self::is_valid_mac(&self.mac_address) {
            return Err(ProvisioningError::InvalidMacAddress(
                self.mac_address.clone(),
            ));
        }

        let dtype = self.normalized_device_type();
        if dtype != "sensor" && dtype != "actuator" {
            return Err(ProvisioningError::InvalidDeviceType(
                self.device_type.clone(),
            ));
        }

        Ok(())
    }

    fn is_valid_mac(mac: &str) -> bool {
        let parts: Vec<&str> = mac.split(':').collect();
        if parts.len() != 6 {
            return false;
        }
        parts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
    }
}

/// Configuration to send to device during adoption
#[derive(Debug, Serialize)]
pub struct DeviceConfig {
    pub user_uuid: String,
    pub device_uuid: String,
    pub device_name: String,
    pub topic: String,
    pub broker_url: String,
    pub wifi_ssid: String,
    pub wifi_password: String,
    pub adopted_status: i32,
    pub adopted_status_desc: String,
}

/// Protocol handler for device communication
pub struct DeviceProtocol {
    conn: SerialConnection,
}

impl DeviceProtocol {
    /// Create a new protocol handler from an open connection
    pub fn new(conn: SerialConnection) -> Self {
        Self { conn }
    }

    /// Send a command and parse the JSON response
    #[instrument(skip(self), fields(cmd = %cmd))]
    async fn send_command<T: for<'de> Deserialize<'de>>(
        &mut self,
        cmd: &str,
        data: Option<serde_json::Value>,
    ) -> Result<T, ProvisioningError> {
        let command = Command {
            cmd: cmd.to_string(),
            data,
        };

        let json = serde_json::to_string(&command)?;
        debug!(json = %json, "sending command");

        let response = self.conn.send_receive(&json).await?;
        debug!(response = %response, "received response");

        if response.is_empty() {
            return Err(ProvisioningError::InvalidResponse("empty response".into()));
        }

        let parsed: T = serde_json::from_str(&response).map_err(|e| {
            warn!(error = %e, response = %response, "failed to parse response");
            ProvisioningError::InvalidJson(e.to_string())
        })?;

        Ok(parsed)
    }

    /// Ping the device to check if it's compatible
    #[instrument(skip(self))]
    pub async fn ping(&mut self) -> Result<PingResponse, ProvisioningError> {
        let response: PingResponse = self.send_command("ping", None).await?;

        if !response.ok {
            return Err(ProvisioningError::DeviceNotCompatible);
        }

        debug!(version = ?response.version, "ping successful");
        Ok(response)
    }

    /// Get device information
    #[instrument(skip(self))]
    pub async fn get_info(&mut self) -> Result<DeviceInfo, ProvisioningError> {
        let info: DeviceInfo = self.send_command("get_info", None).await?;

        info.validate()?;

        debug!(
            mac = %info.mac_address,
            device_type = %info.device_type,
            boarder_type = %info.boarder_type,
            adopted = info.is_adopted(),
            "got device info"
        );

        Ok(info)
    }

    /// Set device configuration (adoption)
    #[instrument(skip(self, config), fields(device_uuid = %config.device_uuid))]
    pub async fn set_config(&mut self, config: DeviceConfig) -> Result<(), ProvisioningError> {
        let data = serde_json::to_value(&config)?;

        let response: GenericResponse = self.send_command("set_config", Some(data)).await?;

        if response.ok != Some(true) {
            let reason = response.error.unwrap_or_else(|| "unknown".to_string());
            return Err(ProvisioningError::CommandFailed {
                cmd: "set_config".to_string(),
                reason,
            });
        }

        debug!("set_config successful");
        Ok(())
    }

    /// Reboot the device
    #[instrument(skip(self))]
    pub async fn reboot(&mut self) -> Result<(), ProvisioningError> {
        let response: GenericResponse = self.send_command("reboot", None).await?;

        if response.ok != Some(true) {
            let reason = response.error.unwrap_or_else(|| "unknown".to_string());
            warn!(reason = %reason, "reboot command returned error (may be expected)");
        }

        debug!("reboot command sent");
        Ok(())
    }

    /// Get the underlying connection's port name
    pub fn port_name(&self) -> &str {
        self.conn.port_name()
    }

    /// Consume self and return the connection
    pub fn into_connection(self) -> SerialConnection {
        self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mac() {
        assert!(DeviceInfo::is_valid_mac("3C:71:BF:4D:DB:0C"));
        assert!(DeviceInfo::is_valid_mac("AA:BB:CC:DD:EE:FF"));
        assert!(!DeviceInfo::is_valid_mac("invalid"));
        assert!(!DeviceInfo::is_valid_mac("3C:71:BF:4D:DB"));
        assert!(!DeviceInfo::is_valid_mac("3C:71:BF:4D:DB:0C:00"));
        assert!(!DeviceInfo::is_valid_mac("GG:HH:II:JJ:KK:LL"));
    }

    #[test]
    fn test_device_info_validation() {
        let valid_sensor = DeviceInfo {
            ok: Some(true),
            adopted_status: 0,
            device_type: "Sensor".to_string(),
            sensor_type: Some("DHT11".to_string()),
            actuator_type: None,
            boarder_type: "ESP32".to_string(),
            mac_address: "3C:71:BF:4D:DB:0C".to_string(),
            device_scale: None,
            parameter_ranges: None,
            command_spec: None,
            firmware_version: None,
            user_uuid: None,
        };
        assert!(valid_sensor.validate().is_ok());

        let invalid_mac = DeviceInfo {
            mac_address: "invalid".to_string(),
            ..valid_sensor.clone()
        };
        assert!(invalid_mac.validate().is_err());

        let invalid_type = DeviceInfo {
            device_type: "unknown".to_string(),
            ..valid_sensor
        };
        assert!(invalid_type.validate().is_err());
    }
}
