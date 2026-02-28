use std::fmt;
use tracing::error;

/// Errors that can occur during device provisioning
#[derive(Debug)]
pub enum ProvisioningError {
    // Connection errors
    PortNotFound(String),
    ConnectionFailed(String),
    PortBusy(String),

    // Communication errors
    ReadTimeout,
    WriteTimeout,
    ReadError(String),
    WriteError(String),

    // Protocol errors
    InvalidResponse(String),
    InvalidJson(String),
    DeviceNotCompatible,
    DeviceAlreadyAdopted,
    CommandFailed { cmd: String, reason: String },

    // Validation errors
    InvalidMacAddress(String),
    InvalidDeviceType(String),

    // Internal errors
    Internal(String),
}

impl fmt::Display for ProvisioningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PortNotFound(port) => write!(f, "Serial port not found: {}", port),
            Self::ConnectionFailed(reason) => write!(f, "Connection failed: {}", reason),
            Self::PortBusy(port) => write!(f, "Serial port is busy: {}", port),

            Self::ReadTimeout => write!(f, "Read timeout"),
            Self::WriteTimeout => write!(f, "Write timeout"),
            Self::ReadError(reason) => write!(f, "Read error: {}", reason),
            Self::WriteError(reason) => write!(f, "Write error: {}", reason),

            Self::InvalidResponse(reason) => write!(f, "Invalid response: {}", reason),
            Self::InvalidJson(reason) => write!(f, "Invalid JSON: {}", reason),
            Self::DeviceNotCompatible => write!(f, "Device is not compatible"),
            Self::DeviceAlreadyAdopted => write!(f, "Device is already adopted"),
            Self::CommandFailed { cmd, reason } => {
                write!(f, "Command '{}' failed: {}", cmd, reason)
            }

            Self::InvalidMacAddress(mac) => write!(f, "Invalid MAC address: {}", mac),
            Self::InvalidDeviceType(dtype) => write!(f, "Invalid device type: {}", dtype),

            Self::Internal(reason) => write!(f, "Internal error: {}", reason),
        }
    }
}

impl std::error::Error for ProvisioningError {}

impl ProvisioningError {
    /// Convert to user-friendly error message
    pub fn to_user_message(&self) -> String {
        match self {
            Self::PortNotFound(_) => "Serial port not found. Check if device is connected.".into(),
            Self::ConnectionFailed(_) => "Failed to connect to device. Try again.".into(),
            Self::PortBusy(_) => "Serial port is busy. Close other applications using it.".into(),

            Self::ReadTimeout => "Device not responding. Check connection.".into(),
            Self::WriteTimeout => "Failed to send data to device.".into(),
            Self::ReadError(_) => "Error reading from device.".into(),
            Self::WriteError(_) => "Error writing to device.".into(),

            Self::InvalidResponse(_) => "Device sent invalid response.".into(),
            Self::InvalidJson(_) => "Device sent malformed data.".into(),
            Self::DeviceNotCompatible => "Device is not compatible with this application.".into(),
            Self::DeviceAlreadyAdopted => {
                "Device is already adopted. Reset the device to adopt again.".into()
            }
            Self::CommandFailed { .. } => "Command failed. Try again.".into(),

            Self::InvalidMacAddress(_) => "Invalid MAC address from device.".into(),
            Self::InvalidDeviceType(_) => "Invalid device type.".into(),

            Self::Internal(_) => "Internal error occurred.".into(),
        }
    }
}

/// Log error and convert to user message
pub fn map_provisioning_error(err: &ProvisioningError) -> String {
    error!(error = %err, "provisioning error");
    err.to_user_message()
}

impl From<std::io::Error> for ProvisioningError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::PortNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::PortBusy(err.to_string()),
            std::io::ErrorKind::TimedOut => Self::ReadTimeout,
            _ => Self::ConnectionFailed(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ProvisioningError {
    fn from(err: serde_json::Error) -> Self {
        Self::InvalidJson(err.to_string())
    }
}

impl From<serialport::Error> for ProvisioningError {
    fn from(err: serialport::Error) -> Self {
        match err.kind {
            serialport::ErrorKind::NoDevice => Self::PortNotFound(err.to_string()),
            serialport::ErrorKind::Io(std::io::ErrorKind::PermissionDenied) => {
                Self::PortBusy(err.to_string())
            }
            _ => Self::ConnectionFailed(err.to_string()),
        }
    }
}
