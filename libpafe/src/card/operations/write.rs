#![allow(dead_code)]

use crate::device::Device;
use crate::protocol::Command;
use crate::protocol::Response;
use crate::types::{BlockData, BlockElement, ServiceCode};
use crate::{Error, Result};

/// Write a single block to the card using WriteWithoutEncryption.
pub fn write_single(
    card: &crate::card::Card,
    device: &mut Device<crate::device::Initialized>,
    service: ServiceCode,
    block: BlockElement,
    data: BlockData,
) -> Result<()> {
    let cmd = Command::WriteWithoutEncryption {
        idm: card.idm,
        service,
        block,
        data,
    };

    let resp = device.execute(cmd, 1000)?;
    match resp {
        Response::WriteWithoutEncryption { statuses, .. } => {
            if statuses.is_empty() {
                return Err(Error::InvalidLength {
                    expected: 11,
                    actual: 0,
                });
            }
            let status = statuses[0];
            if status.0 != 0 || status.1 != 0 {
                Err(Error::FelicaStatus {
                    status1: status.0,
                    status2: status.1,
                })
            } else {
                Ok(())
            }
        }
        _ => Err(Error::PollingFailed),
    }
}

/// Write multiple blocks in a single WriteWithoutEncryption command.
pub fn write_blocks(
    card: &crate::card::Card,
    device: &mut Device<crate::device::Initialized>,
    service: ServiceCode,
    blocks: &[(BlockElement, BlockData)],
) -> Result<()> {
    if blocks.is_empty() {
        return Ok(());
    }

    let services = vec![service];
    let block_elems: Vec<_> = blocks.iter().map(|(b, _)| *b).collect();
    let data_blocks: Vec<_> = blocks.iter().map(|(_, d)| *d).collect();

    let cmd = crate::protocol::commands::Command::WriteWithoutEncryptionMulti {
        idm: card.idm,
        services,
        blocks: block_elems,
        data: data_blocks,
    };

    let resp = device.execute(cmd, 1000)?;
    match resp {
        Response::WriteWithoutEncryption { statuses, .. } => {
            if statuses.is_empty() {
                return Err(Error::InvalidLength {
                    expected: 11,
                    actual: 0,
                });
            }
            // For now, treat the first status as the overall operation status
            let status = statuses[0];
            if status.0 != 0 || status.1 != 0 {
                Err(Error::FelicaStatus {
                    status1: status.0,
                    status2: status.1,
                })
            } else {
                Ok(())
            }
        }
        _ => Err(Error::PollingFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Card;
    use crate::device::Device;
    use crate::protocol::Frame;
    use crate::transport::mock::MockTransport;
    use crate::types::DeviceType;
    use crate::types::{BlockData, BlockElement, Idm, Pmm, ServiceCode, SystemCode};

    #[test]
    fn write_single_success() {
        // Prepare mock transport: init ack + write response
        let mut mock = MockTransport::new(DeviceType::S320);

        // Push a dummy init ack for model initialization
        mock.push_response(vec![0xAA]);

        // Prepare a write response payload: response_code(0x09) + idm + status1 + status2
        let mut payload = vec![0x09];
        let idm_bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
        payload.extend_from_slice(&idm_bytes);
        payload.push(0); // status1
        payload.push(0); // status2

        let frame = Frame::encode(&payload).unwrap();
        mock.push_response(frame);

        // Build device and card
        let boxed: Box<dyn crate::transport::traits::Transport> = Box::new(mock);
        let device = Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes(idm_bytes),
            Pmm::from_bytes([0; 8]),
            SystemCode::new(0x0003),
        );

        // Perform write
        let svc = ServiceCode::new(0x090f);
        let blk = BlockElement::new(0, crate::types::AccessMode::DirectAccessOrRead, 0x0012);
        let data = BlockData::from_bytes([0x5A; 16]);
        write_single(&card, &mut dev, svc, blk, data).unwrap();
    }

    #[test]
    fn write_single_status_error() {
        let mut mock = MockTransport::new(DeviceType::S320);
        mock.push_response(vec![0xAA]);

        let mut payload = vec![0x09];
        let idm_bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
        payload.extend_from_slice(&idm_bytes);
        payload.push(0xA4); // status1 error
        payload.push(0x00); // status2
        let frame = Frame::encode(&payload).unwrap();
        mock.push_response(frame);

        let boxed: Box<dyn crate::transport::traits::Transport> = Box::new(mock);
        let device = Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes(idm_bytes),
            Pmm::from_bytes([0; 8]),
            SystemCode::new(0x0003),
        );

        let svc = ServiceCode::new(0x090f);
        let blk = BlockElement::new(0, crate::types::AccessMode::DirectAccessOrRead, 0x0012);
        let data = BlockData::from_bytes([0x00; 16]);

        match write_single(&card, &mut dev, svc, blk, data) {
            Err(Error::FelicaStatus {
                status1: 0xA4,
                status2: 0x00,
            }) => {}
            other => panic!("expected FelicaStatus, got {:?}", other),
        }
    }

    #[test]
    fn write_blocks_success() {
        let mut mock = crate::transport::mock::MockTransport::new(crate::types::DeviceType::S320);
        mock.push_response(vec![0xAA]);

        let mut payload = vec![0x09];
        let idm_bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
        payload.extend_from_slice(&idm_bytes);
        payload.push(0); // status1
        payload.push(0); // status2
        mock.push_response(crate::protocol::Frame::encode(&payload).unwrap());

        let boxed: Box<dyn crate::transport::traits::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = crate::card::Card::new(
            crate::types::Idm::from_bytes(idm_bytes),
            crate::types::Pmm::from_bytes([0; 8]),
            crate::types::SystemCode::new(0x0003),
        );

        let svc = ServiceCode::new(0x090f);
        let blk1 = BlockElement::new(0, crate::types::AccessMode::DirectAccessOrRead, 0x0012);
        let blk2 = BlockElement::new(0, crate::types::AccessMode::DirectAccessOrRead, 0x0013);
        let data1 = BlockData::from_bytes([0x01; 16]);
        let data2 = BlockData::from_bytes([0x02; 16]);

        write_blocks(&card, &mut dev, svc, &[(blk1, data1), (blk2, data2)]).unwrap();
    }

    #[test]
    fn write_blocks_status_error() {
        let mut mock = crate::transport::mock::MockTransport::new(crate::types::DeviceType::S320);
        mock.push_response(vec![0xAA]);

        let mut payload = vec![0x09];
        let idm_bytes = [1u8, 2, 3, 4, 5, 6, 7, 8];
        payload.extend_from_slice(&idm_bytes);
        payload.push(0xA4);
        payload.push(0x00);
        mock.push_response(crate::protocol::Frame::encode(&payload).unwrap());

        let boxed: Box<dyn crate::transport::traits::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = crate::card::Card::new(
            crate::types::Idm::from_bytes(idm_bytes),
            crate::types::Pmm::from_bytes([0; 8]),
            crate::types::SystemCode::new(0x0003),
        );

        let svc = ServiceCode::new(0x090f);
        let blk = BlockElement::new(0, crate::types::AccessMode::DirectAccessOrRead, 0x0012);
        let data = BlockData::from_bytes([0x00; 16]);

        match write_blocks(&card, &mut dev, svc, &[(blk, data)]) {
            Err(Error::FelicaStatus {
                status1: 0xA4,
                status2: 0x00,
            }) => {}
            other => panic!("expected FelicaStatus, got {:?}", other),
        }
    }
}
