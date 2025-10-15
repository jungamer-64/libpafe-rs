#[path = "../common/mod.rs"]
mod common;

use libpafe::transport::mock::MockTransport;
use libpafe::transport::Transport;
use libpafe::types::DeviceType;

#[test]
fn control_read_failure_and_recovery() {
    let mut m = MockTransport::new(DeviceType::S320);
    common::seed_init_and_frames(&mut m, vec![]);
    m.set_control_failures(1);

    // First control_read should fail (simulated)
    assert!(m.control_read(1000).is_err());

    // Second control_read should return the queued response
    let r = m.control_read(1000).unwrap();
    assert_eq!(r, vec![0xAA]);
}
