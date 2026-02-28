//! Device Provisioning Module
//!
//! Handles device adoption via USB serial connection.
//!
//! ## Architecture
//!
//! ```text
//! serial.rs    → Low-level serial I/O (connect, read, write)
//! protocol.rs  → JSON protocol (ping, get_info, set_config, reboot)
//! adoption.rs  → Business logic (probe, adopt, integrate with DB)
//! error.rs     → Error types and handling
//! ```
//!
//! ## Flow
//!
//! 1. `list_available_ports()` - List USB serial ports
//! 2. `probe_device()` - Connect, ping, get_info
//! 3. `adopt_device()` - set_config, reboot, create in DB

mod adoption;
mod error;
mod protocol;
mod serial;

// Public exports
pub use adoption::{
    adopt_device, get_default_broker_for_adoption, probe_device, AdoptDeviceInput,
    DefaultBrokerInfo, DeviceInfoInput, ProbeDeviceInput, ProbeDeviceResult,
    ProvisioningLogEmitter,
};
pub use error::{map_provisioning_error, ProvisioningError};
pub use protocol::DeviceInfo;
pub use serial::{list_available_ports, BAUDRATES, SerialPortInfo};
