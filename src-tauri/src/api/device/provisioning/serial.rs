use serde::{Deserialize, Serialize};
use serialport::SerialPortType;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;
use tokio_serial::SerialPortBuilderExt;
use tracing::{debug, instrument, warn};

use super::error::ProvisioningError;

// Configuration constants
const DEFAULT_BAUD_RATE: u32 = 115200;
const READ_TIMEOUT_MS: u64 = 5000;
const WRITE_TIMEOUT_MS: u64 = 2000;
const MAX_RESPONSE_SIZE: usize = 4096;

/// Information about an available serial port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialPortInfo {
    pub port_name: String,
    pub port_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
}

/// List available serial ports
#[instrument]
pub fn list_available_ports() -> Result<Vec<SerialPortInfo>, ProvisioningError> {
    let ports = serialport::available_ports()?;

    let result: Vec<SerialPortInfo> = ports
        .into_iter()
        .map(|p| {
            let (port_type, manufacturer, product, serial_number) = match &p.port_type {
                SerialPortType::UsbPort(info) => (
                    "USB".to_string(),
                    info.manufacturer.clone(),
                    info.product.clone(),
                    info.serial_number.clone(),
                ),
                SerialPortType::BluetoothPort => ("Bluetooth".to_string(), None, None, None),
                SerialPortType::PciPort => ("PCI".to_string(), None, None, None),
                SerialPortType::Unknown => ("Unknown".to_string(), None, None, None),
            };

            SerialPortInfo {
                port_name: p.port_name,
                port_type,
                manufacturer,
                product,
                serial_number,
            }
        })
        .collect();

    debug!(count = result.len(), "found serial ports");
    Ok(result)
}

/// Serial connection to a device
pub struct SerialConnection {
    reader: BufReader<tokio::io::ReadHalf<tokio_serial::SerialStream>>,
    writer: tokio::io::WriteHalf<tokio_serial::SerialStream>,
    port_name: String,
}

/// Common baud rates for MicroPython/ESP32 devices
pub const BAUDRATES: [u32; 8] = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];

impl SerialConnection {
    /// Open a connection to the specified serial port
    #[instrument(skip_all, fields(port = %port_name, baud = baud_rate))]
    pub async fn open(port_name: &str, baud_rate: u32) -> Result<Self, ProvisioningError> {
        debug!("opening serial connection");

        let port = tokio_serial::new(port_name, baud_rate)
            .timeout(Duration::from_millis(READ_TIMEOUT_MS))
            .open_native_async()
            .map_err(|e| {
                warn!(error = %e, "failed to open serial port");
                ProvisioningError::ConnectionFailed(e.to_string())
            })?;

        let (reader, writer) = tokio::io::split(port);

        debug!("serial connection opened");

        Ok(Self {
            reader: BufReader::new(reader),
            writer,
            port_name: port_name.to_string(),
        })
    }

    /// Send a line of text to the device
    #[instrument(skip(self, data), fields(port = %self.port_name, len = data.len()))]
    pub async fn write_line(&mut self, data: &str) -> Result<(), ProvisioningError> {
        let line = format!("{}\n", data);

        timeout(Duration::from_millis(WRITE_TIMEOUT_MS), async {
            self.writer.write_all(line.as_bytes()).await?;
            self.writer.flush().await?;
            Ok::<(), std::io::Error>(())
        })
        .await
        .map_err(|_| ProvisioningError::WriteTimeout)?
        .map_err(|e| ProvisioningError::WriteError(e.to_string()))?;

        debug!("wrote line to device");
        Ok(())
    }

    /// Read a line of text from the device
    #[instrument(skip(self), fields(port = %self.port_name))]
    pub async fn read_line(&mut self) -> Result<String, ProvisioningError> {
        let mut response = String::with_capacity(256);

        timeout(Duration::from_millis(READ_TIMEOUT_MS), async {
            loop {
                let bytes_read = self.reader.read_line(&mut response).await?;

                if bytes_read == 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "connection closed",
                    ));
                }

                if response.len() > MAX_RESPONSE_SIZE {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "response too large",
                    ));
                }

                if response.ends_with('\n') {
                    break;
                }
            }
            Ok::<(), std::io::Error>(())
        })
        .await
        .map_err(|_| ProvisioningError::ReadTimeout)?
        .map_err(|e| ProvisioningError::ReadError(e.to_string()))?;

        let response = response.trim().to_string();
        debug!(len = response.len(), "read line from device");
        Ok(response)
    }

    /// Send a command and read the response
    #[instrument(skip(self, data), fields(port = %self.port_name))]
    pub async fn send_receive(&mut self, data: &str) -> Result<String, ProvisioningError> {
        self.write_line(data).await?;
        self.read_line().await
    }

    /// Get the port name
    pub fn port_name(&self) -> &str {
        &self.port_name
    }
}

impl Drop for SerialConnection {
    fn drop(&mut self) {
        debug!(port = %self.port_name, "closing serial connection");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_ports_does_not_panic() {
        let _ = list_available_ports();
    }
}
