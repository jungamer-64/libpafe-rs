// libpafe-rs/libpafe/src/protocol/codec.rs

use crate::Result;

use super::Frame;
use super::commands::Command;
use super::responses::Response;

/// Encode a Command into a full wire frame (with preamble/LCS/DCS/postamble).
pub fn encode_command_frame(cmd: &Command) -> Result<Vec<u8>> {
    let payload = cmd.encode();
    Frame::encode(&payload)
}

/// Decode a full wire frame and parse the contained response for the
/// expected command code.
pub fn decode_response_frame(expected_cmd: u8, frame: &[u8]) -> Result<Response> {
    let payload = Frame::decode(frame)?;
    Response::decode(expected_cmd, &payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Idm, Pmm, SystemCode};
    use proptest::prelude::*;

    #[test]
    fn encode_decode_response_roundtrip() {
        // Build a sample polling response payload and frame
        let mut payload = vec![0x01];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]); // pmm
        payload.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());

        let frame = Frame::encode(&payload).unwrap();
        let resp = decode_response_frame(0x00, &frame).unwrap();

        match resp {
            Response::Polling {
                idm,
                pmm,
                system_code,
            } => {
                assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
                assert_eq!(pmm.as_bytes(), &[9, 10, 11, 12, 13, 14, 15, 16]);
                assert_eq!(system_code.as_u16(), 0x0a0b);
            }
            other => panic!("unexpected response: {:?}", other),
        }
    }

    #[test]
    fn encode_decode_request_service_roundtrip() {
        // Build a sample RequestService response payload and frame
        let mut payload = vec![0x03]; // response code for RequestService (0x02 + 1)
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        payload.push(2); // count
        payload.extend_from_slice(&0x0100u16.to_le_bytes());
        payload.extend_from_slice(&0x0200u16.to_le_bytes());

        let frame = Frame::encode(&payload).unwrap();
        let resp = decode_response_frame(0x02, &frame).unwrap();

        match resp {
            Response::RequestService { idm, versions } => {
                assert_eq!(idm.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
                assert_eq!(versions, vec![0x0100u16, 0x0200u16]);
            }
            other => panic!("unexpected response: {:?}", other),
        }
    }

    #[test]
    fn encode_decode_request_response_roundtrip() {
        let mut payload = vec![0x05]; // response code for RequestResponse (0x04 + 1)
        payload.extend_from_slice(&[9, 9, 9, 9, 9, 9, 9, 9]); // idm
        payload.push(0x02); // mode

        let frame = Frame::encode(&payload).unwrap();
        let resp = decode_response_frame(0x04, &frame).unwrap();

        match resp {
            Response::RequestResponse { idm, mode } => {
                assert_eq!(idm.as_bytes(), &[9, 9, 9, 9, 9, 9, 9, 9]);
                assert_eq!(mode, 0x02);
            }
            other => panic!("unexpected response: {:?}", other),
        }
    }

    // Property test: Decoding random frames with different expected command
    // codes should never panic. Decoders may return Err for malformed or
    // unexpected payloads, but must not panic.
    proptest! {
        #[test]
        fn codec_decode_frame_no_panic(cmd in prop::sample::select(vec![0x00u8,0x06,0x02,0x04,0x0c,0x0a]),
                                        payload in prop::collection::vec(any::<u8>(), 0..64)) {
            use std::panic::{catch_unwind, AssertUnwindSafe};
            let frame = Frame::encode(&payload).unwrap();
            let res = catch_unwind(AssertUnwindSafe(|| decode_response_frame(cmd, &frame)));
            // Should not panic
            prop_assert!(res.is_ok());
        }
    }
}
