// libpafe-rs/libpafe/src/device/models/s330/config.rs

//! S330-specific configuration

/// Control read timeout (ms)
pub const READ_TIMEOUT_MS: u64 = 200;

/// RCS956 RF-ON payload
pub const RCS956_RF_ON: &'static [u8] = &[0xD4u8, 0x32, 0x01, 0x01];

/// RCS956 RF-OFF payload
pub const RCS956_RF_OFF: &'static [u8] = &[0xD4u8, 0x32, 0x01, 0x00];

/// RCS956 GetVersion command
pub const RCS956_GET_VERSION: &'static [u8] = &[0xD4u8, 0x02];

/// RCS956 Deselect command
pub const RCS956_DESELECT: &'static [u8] = &[0xD4u8, 0x44, 0x01];

/// RCS956 InListPassiveTarget (simple default for 106kbps Type A)
pub const RCS956_INLIST_PASSIVE_TARGET: &'static [u8] = &[0xD4u8, 0x4A, 0x01, 0x00];
