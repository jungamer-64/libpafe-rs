#[path = "../common/mod.rs"]
mod common;

use libpafe::types::DeviceType;

#[test]
fn initialized_mock_polling_returns_card() {
    // Prepare responses: init ack, polling frame
    let polling = common::fixtures::polling_frame();
    let responses = vec![vec![0xAA], polling];

    let mut dev = common::helpers::initialized_mock_device(DeviceType::S320, responses).unwrap();

    let card = dev.polling(common::fixtures::sample_system_code()).unwrap();
    assert_eq!(card.idm(), &common::fixtures::sample_idm());
    assert_eq!(card.pmm(), &common::fixtures::sample_pmm());
}
