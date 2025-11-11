// libpafe-rs/libpafe/src/constants.rs
//! Common protocol constants used across the crate

/// `FeliCa` wire frame preamble: `0x00 0x00 0xFF`.
///
/// Source: [Sony FeliCa Technical Specification](https://www.sony.co.jp/en/Products/felica/business/tech-support/)
/// See: [FeliCa Technical Information (overview)](https://www.sony.co.jp/en/Products/felica/business/tech-support/)
/// and [FeliCa System / User's Manual (PDF)](https://www.sony.co.jp/en/Products/felica/business/tech-support/data/fls_usmnl_1.4e.pdf).
/// This constant is a protocol-level value defined by the `FeliCa`
/// `wire framing` rules (`preamble` / `len` / `LCS` / `payload` / `DCS` / `postamble`).
pub const FELICA_PREAMBLE: [u8; 3] = [0x00, 0x00, 0xFF];

/// `FeliCa` wire frame `postamble`: `0x00`.
pub const FELICA_POSTAMBLE: u8 = 0x00;

/// Minimal `FeliCa` `wire frame` length in bytes.
/// This accounts for the preamble (3), length (1), LCS (1), DCS (1) and postamble (1).
pub const FELICA_MIN_FRAME_LEN: usize = 7;

/// Maximum payload length for `FeliCa` frames (in bytes).
pub const FELICA_MAX_PAYLOAD_LEN: usize = 255;

/// PN532/PN533/RCS956 host->device prefix (`D4`) and device->host prefix (`D5`).
///
/// Source: NXP PN532 / PN533 documentation (publicly available).
/// RC-S330 uses RCS956 chip which is PN533-compatible.
/// See: [PN532 datasheet](https://www.nxp.com/docs/en/data-sheet/PN532_C1.pdf),
/// [PN532 User Manual (UM)](https://www.nxp.com/docs/en/user-guide/141520.pdf) and
/// [PN533 User Manual (UM0801)](https://www.nxp.com/docs/en/user-guide/157830_PN533_um080103.pdf).
/// These bytes are part of the `PN53x` framing used when encapsulating
/// `FeliCa` payloads for transport over the `PN53x` controller.
pub const PN532_CMD_PREFIX_HOST: u8 = 0xD4;
/// Device prefix (`D5`) used in device->host PN532/PN533/RCS956 responses.
pub const PN532_CMD_PREFIX_DEVICE: u8 = 0xD5;

/// PN532 `InListPassiveTarget` command / response codes.
/// See: PN532 documentation for `InListPassiveTarget` behaviour and
/// response layout. (RCS956-compatible)
pub const PN532_CMD_INLIST_PASSIVE_TARGET: u8 = 0x4A;
/// Response code returned by PN532/PN533/RCS956 for `InListPassiveTarget`.
/// Typically observed as `0x4B` in device->host response frames.
pub const PN532_RESP_INLIST_PASSIVE_TARGET: u8 = 0x4B;
// libpafe-rs/libpafe/src/constants.rs
