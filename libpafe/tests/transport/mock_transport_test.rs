#[path = "../common/mod.rs"]
mod common;

use libpafe::transport::Transport;
use libpafe::transport::mock::MockTransport;
use libpafe::types::DeviceType;

#[test]
fn mock_transport_send_and_receive() {
    let mut m = MockTransport::new(DeviceType::S310);
    m.push_response(vec![0x01]);
    m.send(&[0xAA]).unwrap();
    assert_eq!(m.sent.len(), 1);
    let r = m.receive(1000).unwrap();
    assert_eq!(r, vec![0x01]);
}

#[test]
fn vendor_control_write_records_call() {
    let mut m = MockTransport::new(DeviceType::S320);
    m.vendor_control_write(0xAB, 0x1234, 0x0001, &[0x10, 0x20])
        .unwrap();
    assert_eq!(m.vendor_calls.len(), 1);
    let (req, val, idx, data) = &m.vendor_calls[0];
    assert_eq!(*req, 0xAB);
    assert_eq!(*val, 0x1234);
    assert_eq!(*idx, 0x0001);
    assert_eq!(data, &vec![0x10u8, 0x20u8]);
}
