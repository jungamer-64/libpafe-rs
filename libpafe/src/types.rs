// libpafe-rs/libpafe/src/types.rs

use crate::Error;
use std::convert::TryFrom;

/// IDm - Newtype Pattern (8 バイト)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Idm([u8; 8]);

impl Idm {
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        crate::utils::bytes_to_hex(self.as_bytes())
    }
}

impl TryFrom<&[u8]> for Idm {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 8 {
            return Err(Error::InvalidLength {
                expected: 8,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        Ok(Self(arr))
    }
}

/// PMm - Newtype Pattern (8 バイト)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pmm([u8; 8]);

impl Pmm {
    pub fn from_bytes(bytes: [u8; 8]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Pmm {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 8 {
            return Err(Error::InvalidLength {
                expected: 8,
                actual: bytes.len(),
            });
        }
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&bytes[..8]);
        Ok(Self(arr))
    }
}

/// SystemCode (u16)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemCode(u16);

impl SystemCode {
    pub const ANY: Self = Self(0xffff);
    pub const COMMON: Self = Self(0xfe00);
    pub const SUICA: Self = Self(0x0003);

    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }

    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }

    pub fn from_le_bytes(bytes: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(bytes))
    }
}

/// ServiceCode (u16)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceCode(u16);

impl ServiceCode {
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }
    pub fn to_le_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }
}

/// BlockData (16 バイト)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockData([u8; 16]);

impl BlockData {
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        crate::utils::bytes_to_hex_spaced(self.as_bytes())
    }

    pub fn to_ascii_safe(&self) -> String {
        self.0
            .iter()
            .map(|&b| {
                if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                }
            })
            .collect()
    }
}

/// DeviceType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    S310,
    S320,
    S330,
}

impl DeviceType {
    pub fn from_product_id(pid: u16) -> Option<Self> {
        match pid {
            0x006c => Some(Self::S310),
            0x01bb => Some(Self::S320),
            0x02e1 => Some(Self::S330),
            _ => None,
        }
    }
}

impl Default for DeviceType {
    fn default() -> Self {
        // Default to S320 as the most common model used during development.
        DeviceType::S320
    }
}

/// AccessMode
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessMode {
    CashBackOrDecrement = 0,
    DirectAccessOrDecrement = 1,
    DirectAccessOrRead = 2,
}

/// BlockElement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockElement {
    pub service_index: u8,
    pub access_mode: AccessMode,
    pub block_number: u16,
}

impl BlockElement {
    pub fn new(service_index: u8, access_mode: AccessMode, block_number: u16) -> Self {
        Self {
            service_index,
            access_mode,
            block_number,
        }
    }

    /// FeliCa のブロック要素を 3 バイトにエンコードする
    pub fn encode(&self) -> [u8; 3] {
        [
            self.service_index,
            self.access_mode as u8,
            (self.block_number & 0xff) as u8,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idm_try_from_ok() {
        let b: [u8; 8] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let idm = Idm::try_from(&b[..]).unwrap();
        assert_eq!(idm.as_bytes(), &b);
    }

    #[test]
    fn idm_try_from_err() {
        let b: [u8; 4] = [0, 1, 2, 3];
        assert!(Idm::try_from(&b[..]).is_err());
    }

    #[test]
    fn block_element_encode_ok() {
        let be = BlockElement::new(1, AccessMode::DirectAccessOrRead, 0x1234);
        assert_eq!(be.encode(), [1, 2, 0x34]);
    }

    #[test]
    fn blockdata_hex_and_ascii() {
        let bytes = [b'a'; 16];
        let block = BlockData::from_bytes(bytes);
        assert!(block.to_hex().len() > 0);
        assert_eq!(block.to_ascii_safe(), "aaaaaaaaaaaaaaaa");
    }

    #[test]
    fn system_and_service_code_roundtrip() {
        let sc = SystemCode::new(0x1234);
        assert_eq!(sc.as_u16(), 0x1234);
        assert_eq!(SystemCode::from_le_bytes(sc.to_le_bytes()).as_u16(), 0x1234);

        let svc = ServiceCode::new(0x090f);
        assert_eq!(svc.as_u16(), 0x090f);
        assert_eq!(svc.to_le_bytes(), 0x090f_u16.to_le_bytes());
    }

    #[test]
    fn device_type_from_pid() {
        assert_eq!(DeviceType::from_product_id(0x006c), Some(DeviceType::S310));
        assert_eq!(DeviceType::from_product_id(0x01bb), Some(DeviceType::S320));
        assert_eq!(DeviceType::from_product_id(0x02e1), Some(DeviceType::S330));
        assert_eq!(DeviceType::from_product_id(0x9999), None);
    }

    #[test]
    fn access_mode_repr_and_block_element_bounds() {
        // Ensure AccessMode discriminants match spec
        assert_eq!(AccessMode::CashBackOrDecrement as u8, 0);
        assert_eq!(AccessMode::DirectAccessOrDecrement as u8, 1);
        assert_eq!(AccessMode::DirectAccessOrRead as u8, 2);

        // BlockElement encodes only low byte of block number per FeliCa spec
        let be = BlockElement::new(2, AccessMode::CashBackOrDecrement, 0x01FF);
        assert_eq!(be.encode(), [2, 0, 0xFF]);
    }

    #[test]
    fn idm_to_hex() {
        let b: [u8; 8] = [0xde, 0xad, 0xbe, 0xef, 0x00, 0x11, 0x22, 0x33];
        let idm = Idm::from_bytes(b);
        assert_eq!(idm.to_hex(), "deadbeef00112233");
    }
}
