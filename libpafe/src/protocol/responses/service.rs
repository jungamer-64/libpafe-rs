// libpafe-rs/libpafe/src/protocol/responses/service.rs

use crate::protocol::parser;
use crate::types::Idm;
use crate::{Error, Result};

/// Decode RequestService response payload (response code = 0x03)
/// Layout: response_code(1) + idm(8) + count(1) + versions(N*2)
pub fn decode_request_service(data: &[u8]) -> Result<(Idm, Vec<u16>)> {
    const MIN_LEN: usize = 1 + 8 + 1; // 10
    parser::ensure_len(data, MIN_LEN)?;

    let expected = 0x02u8 + 1;
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

    let mut versions = Vec::with_capacity(count);
    for i in 0..count {
        let off = 10 + i * 2;
        versions.push(parser::le_u16_at(data, off)?);
    }

    Ok((idm, versions))
}

/// Decode RequestResponse response payload (response code = 0x05)
/// Layout: response_code(1) + idm(8) + mode(1)
pub fn decode_request_response(data: &[u8]) -> Result<(Idm, u8)> {
    const MIN_LEN: usize = 1 + 8 + 1; // 10
    parser::ensure_len(data, MIN_LEN)?;
    let expected = 0x04u8 + 1;
    parser::expect_response_code(data, expected)?;
    let idm = parser::idm_at(data, 1)?;
    let mode = parser::byte_at(data, 9)?;
    Ok((idm, mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_request_service_ok() {
        let mut data = vec![0x03];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        data.push(2);
        data.extend_from_slice(&0x0100u16.to_le_bytes());
        data.extend_from_slice(&0x0200u16.to_le_bytes());

        let (idm, versions) = decode_request_service(&data).unwrap();
        assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(versions, vec![0x0100u16, 0x0200u16]);
    }

    #[test]
    fn decode_request_response_ok() {
        let mut data = vec![0x05];
        data.extend_from_slice(&[9, 9, 9, 9, 9, 9, 9, 9]);
        data.push(0x01);

        let (idm, mode) = decode_request_response(&data).unwrap();
        assert_eq!(idm.as_bytes(), &[9, 9, 9, 9, 9, 9, 9, 9]);
        assert_eq!(mode, 0x01);
    }

    #[test]
    fn decode_request_response_too_short() {
        let data = vec![0x05, 1, 2, 3];
        match decode_request_response(&data) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got {:?}", other),
        }
    }

    #[test]
    fn decode_request_service_too_short() {
        let data: Vec<u8> = vec![];
        match decode_request_service(&data) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got {:?}", other),
        }
    }

    #[test]
    fn decode_request_service_unexpected_response() {
        // Wrong response code (use 0x00 instead of 0x03)
        let data = vec![0x00, 1, 2, 3, 4, 5, 6, 7, 8, 0];
        match decode_request_service(&data) {
            Err(crate::Error::UnexpectedResponse {
                expected: 3,
                actual: 0,
            }) => {}
            other => panic!("expected UnexpectedResponse, got {:?}", other),
        }
    }
}
