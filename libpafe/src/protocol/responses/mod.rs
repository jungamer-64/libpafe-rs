// libpafe-rs/libpafe/src/protocol/responses/mod.rs

pub mod polling;
pub mod read;
pub mod search;
pub mod service;
pub mod system;
pub mod write;

pub use polling::decode_polling;
pub use read::decode_read;
pub use search::decode_search_service_code;
pub use service::{decode_request_response, decode_request_service};
pub use system::decode_request_system_code;
pub use write::decode_write;

/// High-level Response enum. Per-command decoders live in
/// `protocol::responses::<name>.rs` and are dispatched here.
#[derive(Debug, Clone)]
pub enum Response {
    Polling {
        idm: crate::types::Idm,
        pmm: crate::types::Pmm,
        system_code: crate::types::SystemCode,
    },
    ReadWithoutEncryption {
        idm: crate::types::Idm,
        status: (u8, u8),
        blocks: Vec<crate::types::BlockData>,
    },
    WriteWithoutEncryption {
        idm: crate::types::Idm,
        statuses: Vec<(u8, u8)>,
    },
    RequestService {
        idm: crate::types::Idm,
        versions: Vec<u16>,
    },
    RequestResponse {
        idm: crate::types::Idm,
        mode: u8,
    },
    RequestSystemCode {
        idm: crate::types::Idm,
        system_codes: Vec<crate::types::SystemCode>,
    },
    SearchServiceCode {
        idm: crate::types::Idm,
        area_or_service_code: Option<u16>,
    },
}

impl Response {
    /// Decode a response payload (including response code) for the given
    /// expected command code.
    pub fn decode(expected_cmd: u8, data: &[u8]) -> crate::Result<Self> {
        // Fast-fail: ensure at least a response byte is present and the
        // top-level response code matches the expected (command+1). This
        // central check prevents decoders from needing to perform the very
        // first byte verification themselves and avoids accidental panic
        // on empty slices.
        crate::protocol::parser::ensure_len(data, 1)?;
        let expected_response = expected_cmd.wrapping_add(1);
        crate::protocol::parser::expect_response_code(data, expected_response)?;

        match expected_cmd {
            0x00 => {
                let (idm, pmm, sys) = polling::decode_polling(data)?;
                Ok(Self::Polling {
                    idm,
                    pmm,
                    system_code: sys,
                })
            }
            0x06 => {
                let (idm, status, blocks) = read::decode_read(data)?;
                Ok(Self::ReadWithoutEncryption {
                    idm,
                    status,
                    blocks,
                })
            }
            0x08 => {
                let (idm, statuses) = write::decode_write(data)?;
                Ok(Self::WriteWithoutEncryption { idm, statuses })
            }
            0x02 => {
                let (idm, versions) = service::decode_request_service(data)?;
                Ok(Self::RequestService { idm, versions })
            }
            0x04 => {
                let (idm, mode) = service::decode_request_response(data)?;
                Ok(Self::RequestResponse { idm, mode })
            }
            0x0c => {
                let (idm, codes) = system::decode_request_system_code(data)?;
                Ok(Self::RequestSystemCode {
                    idm,
                    system_codes: codes,
                })
            }
            0x0a => {
                let (idm, code) = search::decode_search_service_code(data)?;
                Ok(Self::SearchServiceCode {
                    idm,
                    area_or_service_code: code,
                })
            }
            _ => {
                // Unknown command: report unexpected response using the first
                // byte of the payload if available.
                let actual = data.get(0).copied().unwrap_or(0);
                Err(crate::Error::UnexpectedResponse {
                    expected: expected_cmd + 1,
                    actual,
                })
            }
        }
    }

    /// Return the response code byte associated with this response variant.
    /// This is useful when surfacing `UnexpectedResponse` errors at higher
    /// layers without needing to re-decode the raw payload.
    pub fn response_code(&self) -> u8 {
        match self {
            Response::Polling { .. } => 0x01,
            Response::ReadWithoutEncryption { .. } => 0x07,
            Response::WriteWithoutEncryption { .. } => 0x09,
            Response::RequestService { .. } => 0x03,
            Response::RequestResponse { .. } => 0x05,
            Response::RequestSystemCode { .. } => 0x0D,
            Response::SearchServiceCode { .. } => 0x0B,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SystemCode;
    use proptest::prelude::*;

    #[test]
    fn response_decode_polling_ok() {
        let mut data = vec![0x01];
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        data.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]); // pmm
        data.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());

        match Response::decode(0x00, &data).unwrap() {
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

    // Property test: assert that decoding arbitrary payloads never panics
    // for any known command code. The decoders should return Err for
    // malformed inputs rather than panic.
    proptest! {
        #[test]
        fn response_decode_random_payloads_no_panic(v in prop::collection::vec(any::<u8>(), 0..64)) {
            use std::panic::{catch_unwind, AssertUnwindSafe};
            // List of command codes we support (command -> expected response = +1)
            let cmds = [0x00u8, 0x06u8, 0x08u8, 0x02u8, 0x04u8, 0x0Cu8, 0x0Au8];
            for &cmd in &cmds {
                let res = catch_unwind(AssertUnwindSafe(|| Response::decode(cmd, &v)));
                // Should not panic
                prop_assert!(res.is_ok());
            }
        }
    }
}
