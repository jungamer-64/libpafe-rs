// libpafe-rs/libpafe/src/protocol/commands/search.rs

use crate::types::Idm;

/// Encode SearchServiceCode command (FeliCa command code 0x0A)
/// Layout: command_code(1) + idm(8) + index(2)
pub fn encode_search_service_code(idm: Idm, index: u16) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x0A);
    buf.extend_from_slice(idm.as_bytes());
    buf.extend_from_slice(&index.to_le_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Idm;

    #[test]
    fn encode_search_service_code_basic() {
        let idm = Idm::from_bytes([1, 1, 2, 2, 3, 3, 4, 4]);
        let p = encode_search_service_code(idm, 0x0010);
        let mut expected = vec![0x0A];
        expected.extend_from_slice(&[1, 1, 2, 2, 3, 3, 4, 4]);
        expected.extend_from_slice(&0x0010u16.to_le_bytes());
        assert_eq!(p, expected);
    }
}
