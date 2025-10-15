// libpafe-rs/src/card/mod.rs

use crate::device::Device;
use crate::types::{BlockData, BlockElement, Idm, Pmm, ServiceCode, SystemCode};
use crate::{Error, Result};

mod info;
pub use info::CardInfo;

pub mod builder;
pub mod operations;

pub struct Card {
    idm: Idm,
    pmm: Pmm,
    system_code: SystemCode,
}

impl Card {
    pub fn new(idm: Idm, pmm: Pmm, system_code: SystemCode) -> Self {
        Self {
            idm,
            pmm,
            system_code,
        }
    }

    pub fn idm(&self) -> &Idm {
        &self.idm
    }
    pub fn pmm(&self) -> &Pmm {
        &self.pmm
    }
    pub fn system_code(&self) -> SystemCode {
        self.system_code
    }

    /// Read blocks using ReadWithoutEncryption
    pub fn read_blocks(
        &self,
        device: &mut Device<crate::device::Initialized>,
        services: &[ServiceCode],
        blocks: &[BlockElement],
    ) -> Result<Vec<BlockData>> {
        operations::read_blocks(self, device, services, blocks)
    }

    pub fn read_single(
        &self,
        device: &mut Device<crate::device::Initialized>,
        service: ServiceCode,
        block: u16,
    ) -> Result<BlockData> {
        operations::read_single(self, device, service, block)
    }

    /// Write a single block using WriteWithoutEncryption convenience helper.
    pub fn write_single(
        &self,
        device: &mut Device<crate::device::Initialized>,
        service: ServiceCode,
        block: u16,
        data: BlockData,
    ) -> Result<()> {
        let blk = BlockElement::new(0, crate::types::AccessMode::DirectAccessOrRead, block);
        operations::write::write_single(self, device, service, blk, data)
    }

    /// Write multiple blocks using a single WriteWithoutEncryption command.
    pub fn write_blocks(
        &self,
        device: &mut Device<crate::device::Initialized>,
        service: ServiceCode,
        blocks: &[(BlockElement, BlockData)],
    ) -> Result<()> {
        operations::write::write_blocks(self, device, service, blocks)
    }

    /// Return an iterator over service codes found by SearchServiceCode.
    /// The iterator yields Ok(u16) for each found service, or Err on transport/protocol errors.
    pub fn services<'a>(
        &'a self,
        device: &'a mut Device<crate::device::Initialized>,
    ) -> operations::ServiceIterator<'a> {
        operations::ServiceIterator::new(self, device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;
    use crate::types::DeviceType;
    #[path = "../../tests/common/mod.rs"]
    mod common;

    #[test]
    fn card_read_single_via_device() {
        let mut mock = MockTransport::new(DeviceType::S320);

        // Build a read response: response_code + idm + status1 + status2 + block_count + block(16)
        let mut payload = vec![0x07];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        payload.push(0);
        payload.push(0);
        payload.push(1); // block count
        payload.extend_from_slice(&[0x99; 16]);

        let frame = crate::protocol::Frame::encode(&payload).unwrap();
        // Model initialization will consume one response; provide an init ack
        // first so the read response remains for the read_single call.
        common::seed_init_and_frames(&mut mock, vec![frame]);

        let boxed: Box<dyn crate::transport::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]),
            Pmm::from_bytes([9, 9, 9, 9, 9, 9, 9, 9]),
            SystemCode::new(0x0a0b),
        );

        let block = card
            .read_single(&mut dev, ServiceCode::new(0x090f), 0x0001)
            .unwrap();
        assert_eq!(block.as_bytes(), &[0x99; 16]);
    }

    #[test]
    fn services_iterator_collects_service_codes() {
        let mut mock = MockTransport::new(DeviceType::S320);

        // First response: found service (present=1) with code 0x1111
        let mut p1 = vec![0x0B];
        p1.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        p1.push(1); // present
        p1.extend_from_slice(&0x1111u16.to_le_bytes());
        // Second response: not found (present=0) -> termination
        let mut p2 = vec![0x0B];
        p2.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        p2.push(0);

        // Reserve a handshake ack for model init, then the found/term frames.
        common::seed_init_and_frames(
            &mut mock,
            vec![
                crate::protocol::Frame::encode(&p1).unwrap(),
                crate::protocol::Frame::encode(&p2).unwrap(),
            ],
        );

        let boxed: Box<dyn crate::transport::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]),
            Pmm::from_bytes([9, 9, 9, 9, 9, 9, 9, 9]),
            SystemCode::new(0x0a0b),
        );

        let codes: Vec<_> = card
            .services(&mut dev)
            .collect::<Result<Vec<u16>>>()
            .unwrap();
        assert_eq!(codes, vec![0x1111]);
    }

    #[test]
    fn card_write_single_via_device() {
        let mut mock = crate::transport::mock::MockTransport::new(DeviceType::S320);
        // Init ack + write response
        let mut payload = vec![0x09];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        payload.push(0);
        payload.push(0);
        common::seed_init_and_frames(
            &mut mock,
            vec![crate::protocol::Frame::encode(&payload).unwrap()],
        );

        let boxed: Box<dyn crate::transport::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]),
            Pmm::from_bytes([0; 8]),
            SystemCode::new(0x0003),
        );

        let svc = ServiceCode::new(0x090f);
        let data = BlockData::from_bytes([0x5A; 16]);
        card.write_single(&mut dev, svc, 0x0012, data).unwrap();
    }
}
