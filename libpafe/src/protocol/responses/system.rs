// libpafe-rs/libpafe/src/protocol/responses/system.rs

use crate::protocol::parser;
use crate::types::{Idm, SystemCode};
use crate::{Error, Result};

/// Decode RequestSystemCode response payload (response code = 0x0D)
/// Layout: response_code(1) + idm(8) + count(1) + system_codes(N*2)
pub fn decode_request_system_code(data: &[u8]) -> Result<(Idm, Vec<SystemCode>)> {
    const MIN_LEN: usize = 1 + 8 + 1; // 10
    parser::ensure_len(data, MIN_LEN)?;

    let expected = 0x0C_u8 + 1;
    parser::expect_response_code(data, expected)?;

    let idm = parser::idm_at(data, 1)?;
    let count = parser::byte_at(data, 9)? as usize;
    let needed = 10usize
        .checked_add(count.checked_mul(2).ok_or(Error::InvalidLength {
            expected: 0,
            actual: 0,
        })?)
        .ok_or(Error::InvalidLength {
            expected: 0,
            actual: 0,
        })?;

    parser::ensure_len(data, needed)?;

    let mut codes = Vec::with_capacity(count);
    for i in 0..count {
        let off = 10 + i * 2;
        let sc = parser::le_u16_at(data, off)?;
        codes.push(SystemCode::new(sc));
    }

    Ok((idm, codes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_request_system_code_ok() {
        let mut data = vec![0x0D];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        data.push(2);
        data.extend_from_slice(&0x0011u16.to_le_bytes());
        data.extend_from_slice(&0x0022u16.to_le_bytes());

        let (idm, codes) = decode_request_system_code(&data).unwrap();
        assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(codes[0].as_u16(), 0x0011u16);
        assert_eq!(codes[1].as_u16(), 0x0022u16);
    }

    #[test]
    fn decode_request_system_code_too_short() {
        let data: Vec<u8> = vec![];
        match decode_request_system_code(&data) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got {:?}", other),
        }
    }

    #[test]
    fn decode_request_system_code_unexpected_response() {
        // Wrong response code (use 0x00 instead of expected 0x0D)
        let mut data = vec![0x00];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        data.push(0);
        match decode_request_system_code(&data) {
            Err(crate::Error::UnexpectedResponse {
                expected: 13,
                actual: 0,
            }) => {}
            other => panic!("expected UnexpectedResponse, got {:?}", other),
        }
    }
}
