#[path = "../common/mod.rs"]
mod common;

use libpafe::protocol::Frame;

#[test]
fn polling_frame_payload_matches_fixture() {
    let frame = common::fixtures::polling_frame();
    let payload = Frame::decode(&frame).expect("frame decode");
    let expected = common::fixtures::polling_payload();
    assert_eq!(payload, expected);
}
