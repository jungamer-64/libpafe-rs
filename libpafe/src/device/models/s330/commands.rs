// libpafe-rs/libpafe/src/device/models/s330/commands.rs

//! S330 command helpers

pub fn rcs956_rf_on() -> &'static [u8] {
    // Prefer the RCS956 builder to centralize RCS956/PN533 framing logic.
    super::rcs956::build_rf_on()
}

pub fn rcs956_rf_off() -> &'static [u8] {
    super::config::RCS956_RF_OFF
}

pub fn rcs956_get_version() -> &'static [u8] {
    super::config::RCS956_GET_VERSION
}

pub fn rcs956_deselect() -> &'static [u8] {
    super::config::RCS956_DESELECT
}

/// Build an InListPassiveTarget command for the given max_targets and
/// bit-rate/type parameter. For simple usages callers can use
/// `rcs956_in_list_passive_target_default()`.
pub fn rcs956_in_list_passive_target(max_targets: u8, brty: u8) -> Vec<u8> {
    super::rcs956::build_in_list_passive_target(max_targets, brty)
}

pub fn rcs956_in_list_passive_target_default() -> &'static [u8] {
    super::config::RCS956_INLIST_PASSIVE_TARGET
}
