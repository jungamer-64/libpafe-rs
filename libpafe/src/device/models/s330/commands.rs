// libpafe-rs/libpafe/src/device/models/s330/commands.rs

//! S330 command helpers

pub fn pn533_rf_on() -> &'static [u8] {
    // Prefer the PN533 builder to centralize PN533 framing logic.
    super::pn533::build_rf_on()
}

pub fn pn533_rf_off() -> &'static [u8] {
    super::config::PN533_RF_OFF
}

pub fn pn533_get_version() -> &'static [u8] {
    super::config::PN533_GET_VERSION
}

pub fn pn533_deselect() -> &'static [u8] {
    super::config::PN533_DESELECT
}

/// Build an InListPassiveTarget command for the given max_targets and
/// bit-rate/type parameter. For simple usages callers can use
/// `pn533_in_list_passive_target_default()`.
pub fn pn533_in_list_passive_target(max_targets: u8, brty: u8) -> Vec<u8> {
    super::pn533::build_in_list_passive_target(max_targets, brty)
}

pub fn pn533_in_list_passive_target_default() -> &'static [u8] {
    super::config::PN533_INLIST_PASSIVE_TARGET
}
