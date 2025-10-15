// libpafe-rs/libpafe/src/protocol/parser.rs

use crate::types::Idm;
use crate::{Error, Result};

/// Ensure the slice has at least `min` bytes.
pub fn ensure_len(data: &[u8], min: usize) -> Result<()> {
    if data.len() < min {
        return Err(Error::InvalidLength {
            expected: min,
            actual: data.len(),
        });
    }
    Ok(())
}

/// Read a little-endian u16 at given index, with bounds checking.
pub fn le_u16_at(data: &[u8], idx: usize) -> Result<u16> {
    ensure_len(data, idx + 2)?;
    Ok(u16::from_le_bytes([data[idx], data[idx + 1]]))
}

/// Return a subslice with bounds checking.
pub fn slice_at(data: &[u8], idx: usize, len: usize) -> Result<&[u8]> {
    ensure_len(data, idx + len)?;
    Ok(&data[idx..idx + len])
}

/// Parse an Idm (8 bytes) at `start` index with bounds checking.
pub fn idm_at(data: &[u8], start: usize) -> Result<Idm> {
    let s = slice_at(data, start, 8)?;
    Idm::try_from(s)
}

/// Parse a PMm (8 bytes) at `start` index with bounds checking.
/// PMm is represented by its own newtype `Pmm` but the parsing logic
/// is identical to `idm_at`.
pub fn pmm_at(data: &[u8], start: usize) -> Result<crate::types::Pmm> {
    let s = slice_at(data, start, 8)?;
    crate::types::Pmm::try_from(s)
}

/// Read a single byte at `idx` with bounds checking.
pub fn byte_at(data: &[u8], idx: usize) -> Result<u8> {
    ensure_len(data, idx + 1)?;
    Ok(data[idx])
}

/// Ensure the first byte (response code) equals `expected` and that at
/// least one byte exists in the slice. Returns UnexpectedResponse on mismatch.
pub fn expect_response_code(data: &[u8], expected: u8) -> Result<()> {
    // Use the centralised byte reader which performs bounds checking
    // and returns a uniform error type rather than indexing directly.
    let actual = byte_at(data, 0)?;
    if actual != expected {
        return Err(crate::Error::UnexpectedResponse { expected, actual });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_response_code_ok() {
        let v = vec![0x05u8];
        expect_response_code(&v, 0x05).unwrap();
    }

    #[test]
    fn expect_response_code_mismatch() {
        let v = vec![0x06u8];
        match expect_response_code(&v, 0x05) {
            Err(crate::Error::UnexpectedResponse { expected, actual }) => {
                assert_eq!(expected, 0x05);
                assert_eq!(actual, 0x06);
            }
            other => panic!("expected UnexpectedResponse, got: {:?}", other),
        }
    }

    #[test]
    fn expect_response_code_empty() {
        let v: Vec<u8> = vec![];
        match expect_response_code(&v, 0x05) {
            Err(crate::Error::InvalidLength {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected InvalidLength, got: {:?}", other),
        }
    }
}
