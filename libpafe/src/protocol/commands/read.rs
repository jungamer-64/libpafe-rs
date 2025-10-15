// libpafe-rs/libpafe/src/protocol/commands/read.rs

use crate::types::{BlockElement, Idm, ServiceCode};

/// Encode ReadWithoutEncryption command payload (FeliCa command code 0x06)
pub fn encode_read(idm: Idm, services: &[ServiceCode], blocks: &[BlockElement]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x06); // ReadWithoutEncryption command code
    buf.extend_from_slice(idm.as_bytes());
    buf.push(services.len() as u8);

    for svc in services {
        buf.extend_from_slice(&svc.to_le_bytes());
    }

    buf.push(blocks.len() as u8);
    for blk in blocks {
        buf.extend_from_slice(&blk.encode());
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AccessMode, BlockElement, Idm, ServiceCode};

    #[test]
    fn encode_read_basic() {
        let idm = Idm::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
        let services = [ServiceCode::new(0x090f)];
        let blocks = [BlockElement::new(0, AccessMode::DirectAccessOrRead, 0x0012)];

        let p = encode_read(idm, &services, &blocks);
        // manually build expected
        let mut expected = vec![0x06];
        expected.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        expected.push(1);
        expected.extend_from_slice(&ServiceCode::new(0x090f).to_le_bytes());
        expected.push(1);
        expected.extend_from_slice(&[0, 2, 0x12]);

        assert_eq!(p, expected);
    }
}
