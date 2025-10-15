// libpafe-rs/libpafe/src/constants.rs
//! Common protocol constants used across the crate

/// FeliCa wire frame preamble: 0x00 0x00 0xFF
pub const FELICA_PREAMBLE: [u8; 3] = [0x00, 0x00, 0xFF];

/// FeliCa wire frame postamble: 0x00
pub const FELICA_POSTAMBLE: u8 = 0x00;

/// Minimal FeliCa wire frame length in bytes
pub const FELICA_MIN_FRAME_LEN: usize = 7;

/// Maximum payload length for FeliCa frames
pub const FELICA_MAX_PAYLOAD_LEN: usize = 255;

/// PN532/PN533 host->device prefix (D4) and device->host prefix (D5)
pub const PN532_CMD_PREFIX_HOST: u8 = 0xD4;
pub const PN532_CMD_PREFIX_DEVICE: u8 = 0xD5;

/// PN532 InListPassiveTarget command / response codes
pub const PN532_CMD_INLIST_PASSIVE_TARGET: u8 = 0x4A;
pub const PN532_RESP_INLIST_PASSIVE_TARGET: u8 = 0x4B;
// libpafe-rs/libpafe/src/constants.rs
