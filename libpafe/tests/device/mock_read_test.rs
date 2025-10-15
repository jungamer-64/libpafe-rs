#[path = "../common/mod.rs"]
mod common;

use libpafe::types::{AccessMode, BlockElement, DeviceType};

#[test]
fn read_single_block_via_mock_device() {
    // Prepare responses: init ack, polling frame, read frame
    let polling = common::fixtures::polling_frame();
    let block = common::fixtures::sample_blockdata(0x5A);
    let read = common::fixtures::read_frame_with_block(block.as_bytes());

    let responses = vec![vec![0xAA], polling, read];

    let mut dev = common::helpers::initialized_mock_device(DeviceType::S320, responses).unwrap();

    let card = dev.polling(common::fixtures::sample_system_code()).unwrap();

    let blocks = card
        .read_blocks(
            &mut dev,
            &[common::fixtures::sample_service_code()],
            &[BlockElement::new(0, AccessMode::DirectAccessOrRead, 0x0000)],
        )
        .unwrap();

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0], block);
}
