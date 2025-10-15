// Aggregator for hardware tests. Hardware tests are guarded by the `usb`
// feature so they are only compiled when explicitly requested.

#[cfg(feature = "usb")]
#[path = "hardware/s310_test.rs"]
mod s310_test;

#[cfg(feature = "usb")]
#[path = "hardware/s320_test.rs"]
mod s320_test;

#[cfg(feature = "usb")]
#[path = "hardware/s330_test.rs"]
mod s330_test;
