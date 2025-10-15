#![cfg(feature = "usb")]

#[path = "common.rs"]
mod common;

use libpafe::Result;

// This integration test requires a real RC-S330 connected. It is marked
// `#[ignore]` so CI does not attempt to run it. Run manually with:
//
// cargo test -p libpafe --test s330_test --features usb -- --ignored
//

#[test]
#[ignore]
fn open_and_initialize_s330() -> Result<()> {
    match common::open_and_initialize_device()? {
        Some(_) => Ok(()),
        None => Ok(()),
    }
}
