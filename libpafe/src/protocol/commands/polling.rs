// libpafe-rs/libpafe/src/protocol/commands/polling.rs

use crate::types::SystemCode;
use crate::{Error, Result};

/// Encode Polling command payload (FeliCa command code 0x00)
pub fn encode_polling(system_code: SystemCode, request_code: u8, time_slot: u8) -> Vec<u8> {
    let mut buf = Vec::with_capacity(1 + 2 + 1 + 1);
    buf.push(0x00); // Polling command code
    buf.extend_from_slice(&system_code.to_le_bytes());
    buf.push(request_code);
    buf.push(time_slot);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SystemCode;

    #[test]
    fn encode_polling_basic() {
        let sc = SystemCode::new(0x1234);
        let p = encode_polling(sc, 1, 0);
        assert_eq!(p, vec![0x00, 0x34, 0x12, 1, 0]);
    }
}
