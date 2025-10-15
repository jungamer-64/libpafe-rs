#[path = "../common/mod.rs"]
mod common;

use libpafe::protocol::Command;
use libpafe::types::{AccessMode, BlockElement, SystemCode};

#[test]
fn polling_and_read_encode() {
    let cmd = Command::Polling {
        system_code: SystemCode::new(0x1234),
        request_code: 1,
        time_slot: 0,
    };

    assert_eq!(cmd.command_code(), 0x00);
    assert_eq!(cmd.encode(), vec![0x00, 0x34, 0x12, 1, 0]);

    let idm = common::fixtures::sample_idm();
    let svc = common::fixtures::sample_service_code();
    let block = BlockElement::new(0, AccessMode::DirectAccessOrRead, 0x0012);

    let read_cmd = Command::ReadWithoutEncryption {
        idm,
        services: vec![svc],
        blocks: vec![block],
    };

    let payload = read_cmd.encode();
    // Basic sanity checks on the produced payload
    assert_eq!(payload[0], read_cmd.command_code());
    assert_eq!(&payload[1..9], idm.as_bytes());
    assert_eq!(payload[9], 1); // one service
}
