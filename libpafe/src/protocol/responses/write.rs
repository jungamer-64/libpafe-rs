// libpafe-rs/libpafe/src/protocol/responses/write.rs

use crate::protocol::parser;
use crate::types::Idm;
use crate::{Error, Result};

/// Decode WriteWithoutEncryption response (response code = 0x09).
/// Accepts single or multi-block responses; each block has a 2-byte
/// status tuple (status1, status2). Returns the parsed Idm and the
/// vector of status tuples. If any status is non-zero an immediate
/// `Error::FelicaStatus` is returned.
pub fn decode_write(data: &[u8]) -> Result<(Idm, Vec<(u8, u8)>)> {
    // Minimal: response_code(1) + idm(8) + at least one status pair(2)
    const MIN_LEN: usize = 1 + 8 + 2;
    parser::ensure_len(data, MIN_LEN)?;

    let expected = 0x08u8 + 1;
    parser::expect_response_code(data, expected)?;

    let idm = parser::idm_at(data, 1)?;

    let remaining = data.len().saturating_sub(9);
    if remaining < 2 || remaining % 2 != 0 {
        return Err(Error::InvalidLength {
            expected: MIN_LEN,
            actual: data.len(),
        });
    }

    let count = remaining / 2;
    let mut statuses = Vec::with_capacity(count);
    for i in 0..count {
        let off = 9 + i * 2;
        let s1 = parser::byte_at(data, off)?;
        let s2 = parser::byte_at(data, off + 1)?;
        statuses.push((s1, s2));
    }

    // Surface the first non-zero status as an explicit FelicaStatus
    // For multi-block writes, include the block index in the error.
    for (i, &(s1, s2)) in statuses.iter().enumerate() {
        if s1 != 0 || s2 != 0 {
            if statuses.len() == 1 {
                return Err(Error::FelicaStatus {
                    status1: s1,
                    status2: s2,
                });
            } else {
                return Err(Error::FelicaBlockStatus {
                    index: i,
                    status1: s1,
                    status2: s2,
                });
            }
        }
    }

    Ok((idm, statuses))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_write_ok() {
        let mut data = vec![0x09];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        data.push(0); // status1
        data.push(0); // status2

        let (idm, statuses) = decode_write(&data).unwrap();
        assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(statuses, vec![(0, 0)]);
    }

    #[test]
    fn decode_write_status_error() {
        let mut data = vec![0x09];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        data.push(0xA4);
        data.push(0x00);

        match decode_write(&data) {
            Err(crate::Error::FelicaStatus {
                status1: 0xA4,
                status2: 0x00,
            }) => {}
            other => panic!("expected FelicaStatus, got {:?}", other),
        }
    }

    #[test]
    fn decode_write_too_short() {
        let data = vec![0x09, 1, 2, 3];
        match decode_write(&data) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got {:?}", other),
        }
    }
}
