// libpafe-rs/libpafe/src/error.rs

use thiserror::Error;

/// 共通エラー型
#[derive(Error, Debug)]
pub enum Error {
    #[error("device not found")]
    DeviceNotFound,

    // USB 実装を後から有効化できるように optional dependency にしている
    #[cfg(feature = "usb")]
    #[error("usb error: {0}")]
    Usb(#[from] rusb::Error),

    #[cfg(not(feature = "usb"))]
    #[error("usb error: {0}")]
    UsbString(String),

    #[error("invalid packet length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("felica error: status=({status1:#04x}, {status2:#04x})")]
    FelicaStatus { status1: u8, status2: u8 },
    #[error("felica error at block {index}: status=({status1:#04x}, {status2:#04x})")]
    FelicaBlockStatus {
        index: usize,
        status1: u8,
        status2: u8,
    },

    #[error("checksum mismatch: expected {expected:#04x}, got {actual:#04x}")]
    ChecksumMismatch { expected: u8, actual: u8 },
    #[error("frame format error: {0}")]
    FrameFormat(String),

    #[error("unexpected response code: expected {expected:#04x}, got {actual:#04x}")]
    UnexpectedResponse { expected: u8, actual: u8 },

    #[error("polling failed: no card detected")]
    PollingFailed,

    #[error("operation timed out")]
    Timeout,

    #[error("unsupported operation: {0}")]
    UnsupportedOperation(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_length_display() {
        let err = Error::InvalidLength {
            expected: 8,
            actual: 3,
        };
        let s = format!("{}", err);
        assert!(s.contains("expected 8"));
    }

    #[test]
    fn felica_status_display() {
        let err = Error::FelicaStatus {
            status1: 0xA4,
            status2: 0x00,
        };
        let s = format!("{}", err);
        assert!(s.contains("0xa4"));
        assert!(s.contains("felica error"));
    }

    #[test]
    fn unexpected_response_display() {
        let err = Error::UnexpectedResponse {
            expected: 0x07,
            actual: 0x00,
        };
        let s = format!("{}", err);
        assert!(s.contains("expected 0x07"));
    }

    #[test]
    fn checksum_and_frame_display() {
        let c = Error::ChecksumMismatch {
            expected: 0xFF,
            actual: 0x0F,
        };
        assert!(format!("{}", c).contains("expected 0xff"));

        let f = Error::FrameFormat("bad preamble".to_string());
        assert!(format!("{}", f).contains("bad preamble"));
    }
}
