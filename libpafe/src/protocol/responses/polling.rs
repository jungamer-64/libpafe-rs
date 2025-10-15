// libpafe-rs/libpafe/src/protocol/responses/polling.rs

use crate::protocol::parser;
use crate::types::{Idm, Pmm, SystemCode};
use crate::{Error, Result};

/// Decode a Polling response payload (response code = 0x01)
/// Layout: response_code(1) + idm(8) + pmm(8) + system_code(2)
pub fn decode_polling(data: &[u8]) -> Result<(Idm, Pmm, SystemCode)> {
    const MIN_LEN: usize = 1 + 8 + 8 + 2; // 19
    parser::ensure_len(data, MIN_LEN)?;

    // Ensure we have at least a response byte and it matches expected
    let expected = 0x00u8 + 1;
    parser::expect_response_code(data, expected)?;

    let idm = parser::idm_at(data, 1)?;
    let pmm = parser::pmm_at(data, 9)?;
    let sys = SystemCode::new(parser::le_u16_at(data, 17)?);

    Ok((idm, pmm, sys))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Idm, Pmm, SystemCode};

    #[test]
    fn decode_polling_ok() {
        // Build a sample response: response_code + idm(8) + pmm(8) + system_code(2)
        let mut data = vec![0x01];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        data.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]); // pmm
        data.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());

        let (idm, pmm, sc) = decode_polling(&data).unwrap();
        assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(pmm.as_bytes(), &[9, 10, 11, 12, 13, 14, 15, 16]);
        assert_eq!(sc.as_u16(), 0x0a0b);
    }

    #[test]
    fn decode_polling_too_short() {
        let data: Vec<u8> = vec![];
        match decode_polling(&data) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got {:?}", other),
        }
    }

    #[test]
    fn decode_polling_unexpected_response() {
        // Wrong response code: use 0x00 instead of expected 0x01
        let mut data = vec![0x00];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        data.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]); // pmm
        data.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());

        match decode_polling(&data) {
            Err(crate::Error::UnexpectedResponse {
                expected: 1,
                actual: 0,
            }) => {}
            other => panic!("expected UnexpectedResponse, got {:?}", other),
        }
    }
}
