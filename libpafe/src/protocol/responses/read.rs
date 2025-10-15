// libpafe-rs/libpafe/src/protocol/responses/read.rs

use crate::protocol::parser;
use crate::types::{BlockData, Idm};
use crate::{Error, Result};

/// Decode ReadWithoutEncryption response payload (response code = 0x07)
/// Layout: response_code(1) + idm(8) + status1(1) + status2(1) + block_count(1) + blocks(N*16)
pub fn decode_read(data: &[u8]) -> Result<(Idm, (u8, u8), Vec<BlockData>)> {
    const MIN_LEN: usize = 1 + 8 + 1 + 1 + 1; // 12
    parser::ensure_len(data, MIN_LEN)?;

    let expected = 0x06u8 + 1;
    parser::expect_response_code(data, expected)?;

    let idm = parser::idm_at(data, 1)?;
    let status1 = parser::byte_at(data, 9)?;
    let status2 = parser::byte_at(data, 10)?;

    if status1 != 0 || status2 != 0 {
        return Err(Error::FelicaStatus { status1, status2 });
    }

    let block_count = parser::byte_at(data, 11)? as usize;
    let needed_len = 12usize
        .checked_add(block_count.checked_mul(16).ok_or(Error::InvalidLength {
            expected: 0,
            actual: 0,
        })?)
        .ok_or(Error::InvalidLength {
            expected: 0,
            actual: 0,
        })?;

    parser::ensure_len(data, needed_len)?;

    let mut blocks = Vec::with_capacity(block_count);
    for i in 0..block_count {
        let offset = 12 + i * 16;
        let slice = parser::slice_at(data, offset, 16)?;
        let mut block = [0u8; 16];
        block.copy_from_slice(slice);
        blocks.push(BlockData::from_bytes(block));
    }

    Ok((idm, (status1, status2), blocks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SystemCode;

    #[test]
    fn decode_read_ok() {
        let mut data = vec![0x07];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        data.push(0); // status1
        data.push(0); // status2
        data.push(1); // block_count
        data.extend_from_slice(&[0x41; 16]); // block data

        let (idm, status, blocks) = decode_read(&data).unwrap();
        assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(status, (0, 0));
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].as_bytes(), &[0x41; 16]);
    }

    #[test]
    fn decode_read_unexpected_response() {
        // Response code mismatch: use 0x00 instead of expected 0x07
        let data = vec![0x00, 1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0];
        match decode_read(&data) {
            Err(crate::Error::UnexpectedResponse {
                expected: 7,
                actual: 0,
            }) => {}
            other => panic!("expected UnexpectedResponse, got {:?}", other),
        }
    }

    #[test]
    fn decode_read_status_error() {
        let mut data = vec![0x07];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        data.push(0xA4); // status1 non-zero
        data.push(0x00); // status2
        data.push(0); // block_count

        match decode_read(&data) {
            Err(crate::Error::FelicaStatus {
                status1: 0xA4,
                status2: 0x00,
            }) => {}
            other => panic!("expected FelicaStatus, got {:?}", other),
        }
    }

    #[test]
    fn decode_read_too_short() {
        let data = vec![0x07, 1, 2, 3];
        match decode_read(&data) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got {:?}", other),
        }
    }
}
