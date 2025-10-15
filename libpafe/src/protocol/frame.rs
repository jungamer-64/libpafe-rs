// libpafe-rs/libpafe/src/protocol/frame.rs

use crate::protocol::checksum::{dcs, lcs};
use crate::{Error, Result};

/// FeliCa frame helper. Provides encode/decode of the wire frame
/// Format: [Preamble(3)] [Len(1)] [LCS(1)] [Payload(n)] [DCS(1)] [Postamble(1)]
/// Preamble: 0x00 0x00 0xFF
/// Postamble: 0x00
pub struct Frame {
    pub payload: Vec<u8>,
}

impl Frame {
    /// Encode a payload into a full FeliCa frame
    pub fn encode(payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() > 255 {
            return Err(Error::InvalidLength {
                expected: 255,
                actual: payload.len(),
            });
        }

        let len = payload.len() as u8;
        let mut out = Vec::with_capacity(3 + 1 + 1 + payload.len() + 1 + 1);
        out.extend_from_slice(&crate::constants::FELICA_PREAMBLE);
        out.push(len);
        out.push(lcs(len));
        out.extend_from_slice(payload);
        out.push(dcs(payload));
        out.push(crate::constants::FELICA_POSTAMBLE);
        Ok(out)
    }

    /// Decode a full FeliCa frame and return the payload
    pub fn decode(frame: &[u8]) -> Result<Vec<u8>> {
        // Minimal frame length: preamble(3) + len(1) + lcs(1) + dcs(1) + postamble(1)
        if frame.len() < crate::constants::FELICA_MIN_FRAME_LEN {
            return Err(Error::InvalidLength {
                expected: crate::constants::FELICA_MIN_FRAME_LEN,
                actual: frame.len(),
            });
        }

        if frame[0] != crate::constants::FELICA_PREAMBLE[0]
            || frame[1] != crate::constants::FELICA_PREAMBLE[1]
            || frame[2] != crate::constants::FELICA_PREAMBLE[2]
        {
            return Err(Error::FrameFormat("invalid preamble".into()));
        }

        let len = frame[3];
        let lcs_actual = frame[4];
        let lcs_expected = lcs(len);
        if lcs_actual != lcs_expected {
            return Err(Error::ChecksumMismatch {
                expected: lcs_expected,
                actual: lcs_actual,
            });
        }

        let required_len = 3 + 1 + 1 + (len as usize) + 1 + 1; // FELICA_MIN_FRAME_LEN + len
        if frame.len() != required_len {
            return Err(Error::InvalidLength {
                expected: required_len,
                actual: frame.len(),
            });
        }

        let payload_start = 5usize;
        let payload_end = payload_start + (len as usize);
        let payload = &frame[payload_start..payload_end];

        let dcs_actual = frame[payload_end];
        let dcs_expected = dcs(payload);
        if dcs_actual != dcs_expected {
            return Err(Error::ChecksumMismatch {
                expected: dcs_expected,
                actual: dcs_actual,
            });
        }

        if frame[payload_end + 1] != crate::constants::FELICA_POSTAMBLE {
            return Err(Error::FrameFormat("invalid postamble".into()));
        }

        Ok(payload.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn encode_decode_roundtrip() {
        let payload = vec![0x06, 0x00, 0x12, 0x34];
        let frame = Frame::encode(&payload).unwrap();
        let out = Frame::decode(&frame).unwrap();
        assert_eq!(out, payload);
    }

    proptest! {
        #[test]
        fn frame_encode_decode_roundtrip_prop(payload in prop::collection::vec(any::<u8>(), 0..64)) {
            // Encode then decode should roundtrip for any payload length < 64
            let frame = Frame::encode(&payload).unwrap();
            let decoded = Frame::decode(&frame).unwrap();
            prop_assert_eq!(decoded, payload);
        }
    }

    #[test]
    fn lcs_mismatch() {
        let payload = vec![0x01, 0x02];
        let mut frame = Frame::encode(&payload).unwrap();
        // Corrupt LCS
        frame[4] = frame[4].wrapping_add(1);
        match Frame::decode(&frame) {
            Err(Error::ChecksumMismatch {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected checksum mismatch, got: {:?}", other),
        }
    }

    #[test]
    fn dcs_mismatch() {
        let payload = vec![0x01, 0x02];
        let mut frame = Frame::encode(&payload).unwrap();
        // Corrupt DCS (second last byte)
        let dcs_idx = frame.len() - 2;
        frame[dcs_idx] = frame[dcs_idx].wrapping_add(1);
        match Frame::decode(&frame) {
            Err(Error::ChecksumMismatch {
                expected: _,
                actual: _,
            }) => {}
            other => panic!("expected checksum mismatch, got: {:?}", other),
        }
    }

    #[test]
    fn invalid_preamble() {
        let payload = vec![0x00];
        let mut frame = Frame::encode(&payload).unwrap();
        frame[0] = 0xff;
        match Frame::decode(&frame) {
            Err(Error::FrameFormat(_)) => {}
            other => panic!("expected frame format error, got: {:?}", other),
        }
    }
}
