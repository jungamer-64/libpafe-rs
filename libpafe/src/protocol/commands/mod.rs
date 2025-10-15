// libpafe-rs/libpafe/src/protocol/commands/mod.rs

pub mod polling;
pub mod read;
pub mod search;
pub mod service;
pub mod system;
pub mod write;

pub use polling::encode_polling;
pub use read::encode_read;
pub use search::encode_search_service_code;
pub use service::{encode_request_response, encode_request_service};
pub use system::encode_request_system_code;
pub use write::{encode_write, encode_write_multi};

/// High-level Command enum. New commands should be added here and
/// their per-command encoder placed in `protocol::commands::<name>.rs`.
#[derive(Debug, Clone)]
pub enum Command {
    Polling {
        system_code: crate::types::SystemCode,
        request_code: u8,
        time_slot: u8,
    },
    ReadWithoutEncryption {
        idm: crate::types::Idm,
        services: Vec<crate::types::ServiceCode>,
        blocks: Vec<crate::types::BlockElement>,
    },
    WriteWithoutEncryption {
        idm: crate::types::Idm,
        service: crate::types::ServiceCode,
        block: crate::types::BlockElement,
        data: crate::types::BlockData,
    },
    /// Multi-block write variant
    WriteWithoutEncryptionMulti {
        idm: crate::types::Idm,
        services: Vec<crate::types::ServiceCode>,
        blocks: Vec<crate::types::BlockElement>,
        data: Vec<crate::types::BlockData>,
    },
    RequestService {
        idm: crate::types::Idm,
        node_codes: Vec<u16>,
    },
    RequestResponse {
        idm: crate::types::Idm,
    },
    RequestSystemCode {
        idm: crate::types::Idm,
    },
    SearchServiceCode {
        idm: crate::types::Idm,
        index: u16,
    },
}

impl Command {
    /// Return the command code as defined by FeliCa spec.
    pub fn command_code(&self) -> u8 {
        match self {
            Self::Polling { .. } => 0x00,
            Self::ReadWithoutEncryption { .. } => 0x06,
            Self::WriteWithoutEncryption { .. } => 0x08,
            Self::WriteWithoutEncryptionMulti { .. } => 0x08,
            Self::RequestService { .. } => 0x02,
            Self::RequestResponse { .. } => 0x04,
            Self::RequestSystemCode { .. } => 0x0c,
            Self::SearchServiceCode { .. } => 0x0a,
        }
    }

    /// Encode the command into the raw payload (command code + params).
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Polling {
                system_code,
                request_code,
                time_slot,
            } => encode_polling(*system_code, *request_code, *time_slot),
            Self::ReadWithoutEncryption {
                idm,
                services,
                blocks,
            } => encode_read(*idm, &services[..], &blocks[..]),
            Self::WriteWithoutEncryption {
                idm,
                service,
                block,
                data,
            } => encode_write(*idm, *service, *block, *data),
            Self::WriteWithoutEncryptionMulti {
                idm,
                services,
                blocks,
                data,
            } => encode_write_multi(*idm, &services[..], &blocks[..], &data[..]),
            Self::RequestService { idm, node_codes } => {
                encode_request_service(*idm, &node_codes[..])
            }
            Self::RequestResponse { idm } => encode_request_response(*idm),
            Self::RequestSystemCode { idm } => encode_request_system_code(*idm),
            Self::SearchServiceCode { idm, index } => encode_search_service_code(*idm, *index),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SystemCode;

    #[test]
    fn command_encode_polling() {
        let cmd = Command::Polling {
            system_code: SystemCode::new(0x1234),
            request_code: 1,
            time_slot: 0,
        };

        assert_eq!(cmd.command_code(), 0x00);
        assert_eq!(cmd.encode(), vec![0x00, 0x34, 0x12, 1, 0]);
    }
}
