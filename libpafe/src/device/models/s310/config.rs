//! S310-specific configuration constants

/// Number of attempts to perform the init handshake
pub const ATTEMPTS: usize = 2;

/// Control read timeout (ms) for S310
pub const READ_TIMEOUT_MS: u64 = 200;

/// Vendor init command for S310
pub const INIT_CMD: &'static [u8] = &[0x54u8];

/// Vendor control parameters for S310 init command.
pub const INIT_REQUEST: u8 = 0x01;
pub const INIT_VALUE: u16 = 0;
pub const INIT_INDEX: u16 = 0;
