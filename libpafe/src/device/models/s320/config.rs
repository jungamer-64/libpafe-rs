//! S320-specific configuration

/// Number of init attempts for S320
pub const INIT_ATTEMPTS: usize = 3;

/// Control read timeout (ms)
pub const READ_TIMEOUT_MS: u64 = 200;

/// Init command parts
pub const INIT1: &'static [u8] = &[0x5C, 0x01];
pub const INIT2: &'static [u8] = &[0x5C, 0x02];

/// Vendor control parameters for init commands. These are kept as
/// configuration constants so device models can call explicit vendor
/// control transfers when supported by the transport.
pub const INIT1_REQUEST: u8 = 0x01;
pub const INIT1_VALUE: u16 = 0;
pub const INIT1_INDEX: u16 = 0;

pub const INIT2_REQUEST: u8 = 0x02;
pub const INIT2_VALUE: u16 = 0;
pub const INIT2_INDEX: u16 = 0;
