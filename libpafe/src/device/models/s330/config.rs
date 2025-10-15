// libpafe-rs/libpafe/src/device/models/s330/config.rs

//! S330-specific configuration

/// Control read timeout (ms)
pub const READ_TIMEOUT_MS: u64 = 200;

/// PN533 RF-ON payload
pub const PN533_RF_ON: &'static [u8] = &[0xD4u8, 0x32, 0x01, 0x01];

/// PN533 RF-OFF payload
pub const PN533_RF_OFF: &'static [u8] = &[0xD4u8, 0x32, 0x01, 0x00];

/// PN533 GetVersion command
pub const PN533_GET_VERSION: &'static [u8] = &[0xD4u8, 0x02];

/// PN533 Deselect command
pub const PN533_DESELECT: &'static [u8] = &[0xD4u8, 0x44, 0x01];

/// PN533 InListPassiveTarget (simple default for 106kbps Type A)
pub const PN533_INLIST_PASSIVE_TARGET: &'static [u8] = &[0xD4u8, 0x4A, 0x01, 0x00];
