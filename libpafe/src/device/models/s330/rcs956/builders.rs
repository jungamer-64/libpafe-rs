// libpafe-rs/libpafe/src/device/models/s330/rcs956/builders.rs

//! RCS956 (PN533-compatible) command payload builders

/// Build a simple RCS956/PN533 RF-ON payload. Kept minimal for unit tests and
/// for later expansion when more RCS956 framing is needed.
pub fn build_rf_on() -> &'static [u8] {
    super::super::config::RCS956_RF_ON
}

/// Build a RCS956/PN533 GetVersion command payload.
pub fn build_get_version() -> &'static [u8] {
    super::super::config::RCS956_GET_VERSION
}

/// Build a RCS956/PN533 Deselect command payload.
pub fn build_deselect() -> &'static [u8] {
    super::super::config::RCS956_DESELECT
}

/// Build an InListPassiveTarget command payload. The `brty` parameter
/// encodes the bit-rate / target type as per RCS956/PN533/PN532 conventions
/// (0x00 = 106 kbps type A, 0x01 = 212/424 kbps, etc.).
pub fn build_in_list_passive_target(max_targets: u8, brty: u8) -> Vec<u8> {
    vec![0xD4, 0x4A, max_targets, brty]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_rf_on_is_correct() {
        use crate::device::models::s330::config;
        assert_eq!(build_rf_on(), config::RCS956_RF_ON);
    }

    #[test]
    fn build_get_version_is_correct() {
        use crate::device::models::s330::config;
        assert_eq!(build_get_version(), config::RCS956_GET_VERSION); // Note: constant name kept for compatibility
    }

    #[test]
    fn build_in_list_passive_target_builds_vector() {
        let v = build_in_list_passive_target(1, 0x00);
        assert_eq!(v, vec![0xD4, 0x4A, 0x01, 0x00]);
    }
}
