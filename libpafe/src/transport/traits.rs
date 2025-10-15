// libpafe-rs/libpafe/src/transport/traits.rs

use crate::types::DeviceType;
use crate::{Error, Result};

/// Transport trait abstracts I/O away from protocol/device logic.
pub trait Transport {
    /// Send raw bytes to the device
    fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Receive raw bytes from the device with a timeout in milliseconds
    fn receive(&mut self, timeout_ms: u64) -> Result<Vec<u8>>;

    /// Query the detected device type
    fn device_type(&self) -> Result<DeviceType>;

    /// Perform a transport-level reset
    fn reset(&mut self) -> Result<()>;

    /// Perform a vendor-specific control write. Default implementation
    /// falls back to `send` so existing transports continue to work.
    fn control_write(&mut self, data: &[u8]) -> Result<()> {
        self.send(data)
    }

    /// Perform a vendor-specific control read with timeout. Default
    /// implementation falls back to `receive`.
    fn control_read(&mut self, timeout_ms: u64) -> Result<Vec<u8>> {
        self.receive(timeout_ms)
    }

    /// Vendor-specific control write that allows specifying the USB
    /// `request`/`value`/`index` fields. Default falls back to
    /// `control_write` for transports that do not support explicit
    /// control transfer parameters.
    fn vendor_control_write(
        &mut self,
        _request: u8,
        _value: u16,
        _index: u16,
        data: &[u8],
    ) -> Result<()> {
        self.control_write(data)
    }

    /// Vendor-specific control read with explicit request/value/index and
    /// timeout. Default falls back to `control_read`.
    fn vendor_control_read(
        &mut self,
        _request: u8,
        _value: u16,
        _index: u16,
        timeout_ms: u64,
    ) -> Result<Vec<u8>> {
        self.control_read(timeout_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;

    #[test]
    fn trait_object_send_receive() {
        let mut m = MockTransport::new(crate::types::DeviceType::S320);
        m.push_response(vec![0x01, 0x02]);
        m.send(&[0x10]).unwrap();
        let r = m.receive(1000).unwrap();
        assert_eq!(r, vec![0x01, 0x02]);
        assert_eq!(m.device_type().unwrap(), crate::types::DeviceType::S320);
    }

    #[test]
    fn vendor_control_default_uses_control() {
        let mut m = MockTransport::new(crate::types::DeviceType::S320);
        // vendor_control_read should fallback to control_read and return pre-seeded response
        m.push_response(vec![0x99]);
        let r = m.vendor_control_read(0x01, 0, 0, 1000).unwrap();
        assert_eq!(r, vec![0x99]);
    }
}
