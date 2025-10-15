// libpafe-rs/libpafe/src/protocol/responses/search.rs

use crate::error::{Error, Result};
use crate::protocol::parser;
use crate::types::Idm;

/// Decode SearchServiceCode response (expected response code = 0x0B)
/// Layout assumed: response_code(1) + idm(8) + present_flag(1) + optional service_code(2)
pub fn decode_search_service_code(data: &[u8]) -> Result<(Idm, Option<u16>)> {
    const MIN_LEN: usize = 1 + 8 + 1; // resp + idm + present_flag
    parser::ensure_len(data, MIN_LEN)?;
    // check response code is the expected (0x0A command => 0x0B response)
    parser::expect_response_code(data, 0x0B)?;
    let idm = parser::idm_at(data, 1)?;
    let present_flag = parser::byte_at(data, 9)?;
    if present_flag == 0 {
        Ok((idm, None))
    } else {
        parser::ensure_len(data, MIN_LEN + 2)?; // must have 2 bytes for service code
        let code = parser::le_u16_at(data, 10)?;
        Ok((idm, Some(code)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Idm;

    #[test]
    fn decode_search_service_code_not_found() {
        // resp + idm + present=0
        let mut v = vec![0x0B];
        v.extend_from_slice(&[9, 8, 7, 6, 5, 4, 3, 2]);
        v.push(0);
        let (idm, maybe) = decode_search_service_code(&v).expect("decode");
        assert_eq!(idm.as_bytes(), &[9, 8, 7, 6, 5, 4, 3, 2]);
        assert!(maybe.is_none());
    }

    #[test]
    fn decode_search_service_code_found() {
        let mut v = vec![0x0B];
        v.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        v.push(1);
        v.extend_from_slice(&0x1234u16.to_le_bytes());
        let (idm, maybe) = decode_search_service_code(&v).expect("decode");
        assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(maybe, Some(0x1234));
    }

    #[test]
    fn decode_search_service_code_too_short() {
        let data: Vec<u8> = vec![];
        match decode_search_service_code(&data) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got {:?}", other),
        }
    }
}
