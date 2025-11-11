// libpafe-rs/libpafe/src/device/models/s330/rcs956/mod.rs

//! RCS956 (PN533-compatible) chip helper utilities for RC-S330

mod builders;
mod extractor;
mod multi_frame;

pub use builders::{build_in_list_passive_target, build_rf_on};
pub use extractor::extract_felica_from_pn532_response;
pub use multi_frame::extract_all_felica_frames_from_pn532_response;

// Not currently used publicly, but kept for future expansion
#[allow(unused)]
use builders::{build_deselect, build_get_version};
