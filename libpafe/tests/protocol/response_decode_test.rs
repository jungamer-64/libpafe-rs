#[path = "../common/mod.rs"]
mod common;

use libpafe::protocol::{Frame, Response};

#[test]
fn polling_response_decodes_to_polling_variant() {
    let frame = common::fixtures::polling_frame();
    let payload = Frame::decode(&frame).unwrap();
    let resp = Response::decode(0x00, &payload).unwrap();
    match resp {
        Response::Polling {
            idm,
            pmm,
            system_code,
        } => {
            assert_eq!(idm, common::fixtures::sample_idm());
            assert_eq!(pmm, common::fixtures::sample_pmm());
            assert_eq!(system_code, common::fixtures::sample_system_code());
        }
        other => panic!("expected polling response, got {:?}", other),
    }
}

#[test]
fn read_response_decodes_blocks() {
    let block = common::fixtures::sample_blockdata(0xAA);
    let frame = common::fixtures::read_frame_with_block(block.as_bytes());
    let payload = Frame::decode(&frame).unwrap();
    let resp = Response::decode(0x06, &payload).unwrap();

    match resp {
        Response::ReadWithoutEncryption {
            idm: _,
            status,
            blocks,
        } => {
            assert_eq!(status, (0, 0));
            assert_eq!(blocks.len(), 1);
            assert_eq!(blocks[0], block);
        }
        other => panic!("expected read response, got {:?}", other),
    }
}
