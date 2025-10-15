// libpafe-rs/libpafe/src/protocol/commands/service.rs

use crate::types::Idm;

/// Encode RequestService command (FeliCa command code 0x02)
/// Layout: command_code(1) + idm(8) + node_count(1) + node_code_list(2*N)
pub fn encode_request_service(idm: Idm, node_codes: &[u16]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x02);
    buf.extend_from_slice(idm.as_bytes());
    buf.push(node_codes.len() as u8);
    for n in node_codes {
        buf.extend_from_slice(&n.to_le_bytes());
    }
    buf
}

/// Encode RequestResponse command (FeliCa command code 0x04)
/// Layout: command_code(1) + idm(8)
pub fn encode_request_response(idm: Idm) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.push(0x04);
    buf.extend_from_slice(idm.as_bytes());
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Idm;

    #[test]
    fn encode_request_service_basic() {
        let idm = Idm::from_bytes([1, 2, 3, 4, 5, 6, 7, 8]);
        let nodes = [0x1001u16, 0x1002u16];
        let p = encode_request_service(idm, &nodes);
        let mut expected = vec![0x02];
        expected.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        expected.push(2);
        expected.extend_from_slice(&0x1001u16.to_le_bytes());
        expected.extend_from_slice(&0x1002u16.to_le_bytes());
        assert_eq!(p, expected);
    }

    #[test]
    fn encode_request_response_basic() {
        let idm = Idm::from_bytes([9, 9, 9, 9, 9, 9, 9, 9]);
        let p = encode_request_response(idm);
        let mut expected = vec![0x04];
        expected.extend_from_slice(&[9, 9, 9, 9, 9, 9, 9, 9]);
        assert_eq!(p, expected);
    }
}
