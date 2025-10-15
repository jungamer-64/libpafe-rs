//! S310-specific command helpers

pub fn vendor_init() -> &'static [u8] {
    super::config::INIT_CMD
}
