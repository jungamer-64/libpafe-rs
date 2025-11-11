// libpafe-rs/src/card/mod.rs

use crate::Result;
use crate::device::Device;
use crate::types::{
    Atqb, BlockData, BlockElement, CardType, Idm, Pmm, ServiceCode, SystemCode, Uid,
};

mod info;
pub use info::CardInfo;

pub mod builder;
pub mod operations;

/// Card represents a detected NFC card, which can be FeliCa (Type F) or Type A/B
#[derive(Debug, Clone)]
pub enum Card {
    /// FeliCa / Type F card with IDm, PMm, and System Code
    TypeF {
        idm: Idm,
        pmm: Pmm,
        system_code: SystemCode,
    },
    /// Type A card with UID
    TypeA { uid: Uid },
    /// Type B card with UID and ATQB
    TypeB { uid: Uid, atqb: Atqb },
}

impl Card {
    /// Create a new FeliCa (Type F) card
    pub fn new(idm: Idm, pmm: Pmm, system_code: SystemCode) -> Self {
        Self::TypeF {
            idm,
            pmm,
            system_code,
        }
    }

    /// Create a new Type F card (alias for new)
    pub fn new_type_f(idm: Idm, pmm: Pmm, system_code: SystemCode) -> Self {
        Self::TypeF {
            idm,
            pmm,
            system_code,
        }
    }

    /// Create a new Type A card
    pub fn new_type_a(uid: Uid) -> Self {
        Self::TypeA { uid }
    }

    /// Create a new Type B card
    pub fn new_type_b(uid: Uid, atqb: Atqb) -> Self {
        Self::TypeB { uid, atqb }
    }

    /// Get card type
    pub fn card_type(&self) -> CardType {
        match self {
            Card::TypeF { .. } => CardType::TypeF,
            Card::TypeA { .. } => CardType::TypeA,
            Card::TypeB { .. } => CardType::TypeB,
        }
    }

    /// Get IDm (FeliCa only, returns None for Type A/B)
    pub fn idm(&self) -> Option<&Idm> {
        match self {
            Card::TypeF { idm, .. } => Some(idm),
            _ => None,
        }
    }

    /// Get PMm (FeliCa only, returns None for Type A/B)
    pub fn pmm(&self) -> Option<&Pmm> {
        match self {
            Card::TypeF { pmm, .. } => Some(pmm),
            _ => None,
        }
    }

    /// Get System Code (FeliCa only, returns None for Type A/B)
    pub fn system_code(&self) -> Option<SystemCode> {
        match self {
            Card::TypeF { system_code, .. } => Some(*system_code),
            _ => None,
        }
    }

    /// Get UID (Type A/B only, returns None for FeliCa)
    pub fn uid(&self) -> Option<&Uid> {
        match self {
            Card::TypeA { uid } | Card::TypeB { uid, .. } => Some(uid),
            _ => None,
        }
    }

    /// Get ATQB (Type B only, returns None for Type A/F)
    pub fn atqb(&self) -> Option<&Atqb> {
        match self {
            Card::TypeB { atqb, .. } => Some(atqb),
            _ => None,
        }
    }

    /// Read blocks using ReadWithoutEncryption (FeliCa/Type F only)
    pub fn read_blocks(
        &self,
        device: &mut Device<crate::device::Initialized>,
        services: &[ServiceCode],
        blocks: &[BlockElement],
    ) -> Result<Vec<BlockData>> {
        if !matches!(self, Card::TypeF { .. }) {
            return Err(crate::Error::UnsupportedOperation(
                "read_blocks is only supported for FeliCa (Type F) cards".into(),
            ));
        }
        operations::read_blocks(self, device, services, blocks)
    }

    /// Read a single block (FeliCa/Type F only)
    pub fn read_single(
        &self,
        device: &mut Device<crate::device::Initialized>,
        service: ServiceCode,
        block: u16,
    ) -> Result<BlockData> {
        if !matches!(self, Card::TypeF { .. }) {
            return Err(crate::Error::UnsupportedOperation(
                "read_single is only supported for FeliCa (Type F) cards".into(),
            ));
        }
        operations::read_single(self, device, service, block)
    }

    /// Write a single block using WriteWithoutEncryption (FeliCa/Type F only)
    pub fn write_single(
        &self,
        device: &mut Device<crate::device::Initialized>,
        service: ServiceCode,
        block: u16,
        data: BlockData,
    ) -> Result<()> {
        if !matches!(self, Card::TypeF { .. }) {
            return Err(crate::Error::UnsupportedOperation(
                "write_single is only supported for FeliCa (Type F) cards".into(),
            ));
        }
        let blk = BlockElement::new(0, crate::types::AccessMode::DirectAccessOrRead, block);
        operations::write::write_single(self, device, service, blk, data)
    }

    /// Write multiple blocks using a single WriteWithoutEncryption command (FeliCa/Type F only)
    pub fn write_blocks(
        &self,
        device: &mut Device<crate::device::Initialized>,
        service: ServiceCode,
        blocks: &[(BlockElement, BlockData)],
    ) -> Result<()> {
        if !matches!(self, Card::TypeF { .. }) {
            return Err(crate::Error::UnsupportedOperation(
                "write_blocks is only supported for FeliCa (Type F) cards".into(),
            ));
        }
        operations::write::write_blocks(self, device, service, blocks)
    }

    /// Return an iterator over service codes found by SearchServiceCode (FeliCa/Type F only)
    pub fn services<'a>(
        &'a self,
        device: &'a mut Device<crate::device::Initialized>,
    ) -> operations::ServiceIterator<'a> {
        operations::ServiceIterator::new(self, device)
    }

    /// Request service/node key versions for the provided codes (FeliCa only)
    pub fn request_service_versions(
        &self,
        device: &mut Device<crate::device::Initialized>,
        node_codes: &[u16],
    ) -> Result<Vec<u16>> {
        if !matches!(self, Card::TypeF { .. }) {
            return Err(crate::Error::UnsupportedOperation(
                "request_service_versions is only supported for FeliCa (Type F) cards".into(),
            ));
        }
        operations::request_service_versions(self, device, node_codes)
    }

    /// Query the card's current operating mode via RequestResponse (FeliCa only)
    pub fn request_response_mode(
        &self,
        device: &mut Device<crate::device::Initialized>,
    ) -> Result<u8> {
        if !matches!(self, Card::TypeF { .. }) {
            return Err(crate::Error::UnsupportedOperation(
                "request_response_mode is only supported for FeliCa (Type F) cards".into(),
            ));
        }
        operations::request_response_mode(self, device)
    }

    /// Request the list of published system codes (FeliCa only)
    pub fn request_system_codes(
        &self,
        device: &mut Device<crate::device::Initialized>,
    ) -> Result<Vec<SystemCode>> {
        if !matches!(self, Card::TypeF { .. }) {
            return Err(crate::Error::UnsupportedOperation(
                "request_system_codes is only supported for FeliCa (Type F) cards".into(),
            ));
        }
        operations::request_system_codes(self, device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support as common;
    use crate::transport::mock::MockTransport;
    use crate::types::DeviceType;

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

    #[test]
    fn card_request_service_versions_via_device() {
        let mut mock = MockTransport::new(DeviceType::S320);
        let idm = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut payload = vec![0x03];
        payload.extend_from_slice(&idm);
        payload.push(2); // two services
        payload.extend_from_slice(&0x0100u16.to_le_bytes());
        payload.extend_from_slice(&0x0200u16.to_le_bytes());
        common::seed_init_and_frames(
            &mut mock,
            vec![crate::protocol::Frame::encode(&payload).unwrap()],
        );

        let boxed: Box<dyn crate::transport::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes(idm),
            Pmm::from_bytes([0; 8]),
            SystemCode::new(0x0a0b),
        );

        let versions = card
            .request_service_versions(&mut dev, &[0x1000, 0x1001])
            .unwrap();
        assert_eq!(versions, vec![0x0100, 0x0200]);
    }

    #[test]
    fn card_request_response_mode_via_device() {
        let mut mock = MockTransport::new(DeviceType::S320);
        let idm = [9, 9, 9, 9, 9, 9, 9, 9];
        let mut payload = vec![0x05];
        payload.extend_from_slice(&idm);
        payload.push(0x01); // mode
        common::seed_init_and_frames(
            &mut mock,
            vec![crate::protocol::Frame::encode(&payload).unwrap()],
        );

        let boxed: Box<dyn crate::transport::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes(idm),
            Pmm::from_bytes([0; 8]),
            SystemCode::new(0x0003),
        );

        let mode = card.request_response_mode(&mut dev).unwrap();
        assert_eq!(mode, 0x01);
    }

    #[test]
    fn card_request_system_codes_via_device() {
        let mut mock = MockTransport::new(DeviceType::S320);
        let idm = [3, 4, 5, 6, 7, 8, 9, 0x10];
        let mut payload = vec![0x0D];
        payload.extend_from_slice(&idm);
        payload.push(2); // two system codes
        payload.extend_from_slice(&SystemCode::SUICA.to_le_bytes());
        payload.extend_from_slice(&SystemCode::COMMON.to_le_bytes());
        common::seed_init_and_frames(
            &mut mock,
            vec![crate::protocol::Frame::encode(&payload).unwrap()],
        );

        let boxed: Box<dyn crate::transport::Transport> = Box::new(mock);
        let device = crate::device::Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = Card::new(
            Idm::from_bytes(idm),
            Pmm::from_bytes([0; 8]),
            SystemCode::new(0xffff),
        );

        let codes = card.request_system_codes(&mut dev).unwrap();
        let collected: Vec<u16> = codes.iter().map(SystemCode::as_u16).collect();
        assert_eq!(
            collected,
            vec![SystemCode::SUICA.as_u16(), SystemCode::COMMON.as_u16()]
        );
    }
}
