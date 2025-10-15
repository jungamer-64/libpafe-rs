// libpafe-rs/libpafe/src/protocol/commands/write.rs

use crate::types::{BlockData, BlockElement, Idm, ServiceCode};

/// Encode WriteWithoutEncryption command payload (FeliCa command code 0x08)
/// Layout (single-block variant):
/// command_code(1) + idm(8) + number_of_services(1) + service_code_list(2*N) + number_of_blocks(1) + block_list(3*N) + block_data(16*N)
pub fn encode_write(
    idm: Idm,
    service: ServiceCode,
    block: BlockElement,
    data: BlockData,
) -> Vec<u8> {
    // Delegate to the multi-block encoder for single-block cases
    encode_write_multi(idm, &[service], &[block], &[data])
}

fn block_data_to_bytes(b: &BlockData) -> [u8; 16] {
    *b.as_bytes()
}

/// Encode multi-block WriteWithoutEncryption command payload
/// Layout: command_code(1) + idm(8) + service_count(1) + service_code_list(2*N)
///         + block_count(1) + block_list(3*M) + block_data(16*M)
pub fn encode_write_multi(
    idm: Idm,
    services: &[ServiceCode],
    blocks: &[BlockElement],
    data_blocks: &[BlockData],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x08); // WriteWithoutEncryption command code
    buf.extend_from_slice(idm.as_bytes());

    // service list
    buf.push(services.len() as u8);
    for svc in services {
        buf.extend_from_slice(&svc.to_le_bytes());
    }

    // block list
    buf.push(blocks.len() as u8);
    for blk in blocks {
        buf.extend_from_slice(&blk.encode());
    }

    // block data (each 16 bytes)
    for db in data_blocks {
        buf.extend_from_slice(&block_data_to_bytes(db));
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AccessMode, BlockData, BlockElement, Idm, ServiceCode};

    #[test]
    fn encode_write_single_block() {
        let idm = Idm::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
        let svc = ServiceCode::new(0x090f);
        let blk = BlockElement::new(0, AccessMode::DirectAccessOrRead, 0x0012);
        let data = BlockData::from_bytes([0x5A; 16]);

        let p = encode_write(idm, svc, blk, data);

        let mut expected = vec![0x08];
        expected.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        expected.push(1);
        expected.extend_from_slice(&svc.to_le_bytes());
        expected.push(1);
        expected.extend_from_slice(&blk.encode());
        expected.extend_from_slice(&[0x5A; 16]);

        assert_eq!(p, expected);
    }

    #[test]
    fn encode_write_multi_block() {
        let idm = Idm::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
        let svc = ServiceCode::new(0x090f);
        let blk1 = BlockElement::new(0, AccessMode::DirectAccessOrRead, 0x0012);
        let blk2 = BlockElement::new(0, AccessMode::DirectAccessOrRead, 0x0013);
        let d1 = BlockData::from_bytes([0xAA; 16]);
        let d2 = BlockData::from_bytes([0xBB; 16]);

        let p = encode_write_multi(idm, &[svc], &[blk1, blk2], &[d1, d2]);

        // Basic sanity checks: starts with command code and contains two data blocks
        assert_eq!(p[0], 0x08);
        assert!(p.windows(16).any(|w| w == [0xAA; 16]));
        assert!(p.windows(16).any(|w| w == [0xBB; 16]));
    }
}
