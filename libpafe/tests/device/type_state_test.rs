#[path = "../common/mod.rs"]
mod common;

use libpafe::device::Device;
use libpafe::transport::mock::MockTransport;
use libpafe::types::DeviceType;

#[test]
fn initialize_transitions_and_device_type() {
    let mut m = MockTransport::new(DeviceType::S320);
    // Model init ack
    common::seed_init_and_frames(&mut m, vec![]);

    let boxed: Box<dyn libpafe::transport::traits::Transport> = Box::new(m);
    let device = Device::new_with_transport(boxed).unwrap();

    // Uninitialized device exposes device_type
    assert_eq!(device.device_type(), DeviceType::S320);

    // Transition to initialized
    let initialized = device.initialize().unwrap();
    assert_eq!(initialized.device_type(), DeviceType::S320);
}
