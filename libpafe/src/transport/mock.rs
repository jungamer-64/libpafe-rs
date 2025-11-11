// libpafe-rs/libpafe/src/transport/mock.rs

use crate::transport::traits::Transport;
use crate::types::DeviceType;
use crate::{Error, Result};

/// Mock transport for unit tests. It records sent payloads and returns queued responses.
#[derive(Debug, Default)]
pub struct MockTransport {
    pub sent: Vec<Vec<u8>>,
    pub responses: Vec<Vec<u8>>,
    pub device_type: DeviceType,
    /// Testing hook: number of control_read calls that should fail with Timeout
    pub control_failures: usize,
    /// Record vendor control write calls: (request, value, index, data)
    pub vendor_calls: Vec<(u8, u16, u16, Vec<u8>)>,
    /// Record vendor control read calls: (request, value, index)
    pub vendor_reads: Vec<(u8, u16, u16)>,
}

impl MockTransport {
    pub fn new(device_type: DeviceType) -> Self {
        Self {
            sent: Vec::new(),
            responses: Vec::new(),
            device_type,
            control_failures: 0,
            vendor_calls: Vec::new(),
            vendor_reads: Vec::new(),
        }
    }

    /// Set how many subsequent control_read calls should fail (for tests).
    pub fn set_control_failures(&mut self, n: usize) {
        self.control_failures = n;
    }

    pub fn push_response(&mut self, resp: Vec<u8>) {
        self.responses.push(resp);
    }

    pub fn pop_sent(&mut self) -> Option<Vec<u8>> {
        self.sent.pop()
    }
}

impl Transport for MockTransport {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        self.sent.push(data.to_vec());
        Ok(())
    }

    fn receive(&mut self, _timeout_ms: u64) -> Result<Vec<u8>> {
        if self.responses.is_empty() {
            Err(Error::Timeout)
        } else {
            Ok(self.responses.remove(0))
        }
    }

    fn device_type(&self) -> Result<DeviceType> {
        Ok(self.device_type)
    }

    fn reset(&mut self) -> Result<()> {
        // Reset should clear recorded sent messages but preserve queued
        // responses so unit tests can pre-seed expected replies (handshake
        // ACKs, frames) before handing the transport to a Device.
        self.sent.clear();
        Ok(())
    }

    fn control_write(&mut self, data: &[u8]) -> Result<()> {
        // For tests, control_write behaves like send but is recorded
        // separately in the same `sent` log.
        self.sent.push(data.to_vec());
        Ok(())
    }

    fn vendor_control_write(
        &mut self,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) -> Result<()> {
        // Record the explicit vendor control parameters for assertions in tests
        self.vendor_calls
            .push((request, value, index, data.to_vec()));
        // Also record the payload in sent for backwards compatibility tests
        self.sent.push(data.to_vec());
        Ok(())
    }

    fn control_read(&mut self, _timeout_ms: u64) -> Result<Vec<u8>> {
        // Simulate control read failures when configured by tests.
        if self.control_failures > 0 {
            self.control_failures -= 1;
            return Err(Error::Timeout);
        }
        // For tests, reuse the queued responses to simulate control reads.
        self.receive(_timeout_ms)
    }

    fn vendor_control_read(
        &mut self,
        request: u8,
        value: u16,
        index: u16,
        timeout_ms: u64,
    ) -> Result<Vec<u8>> {
        // Record call parameters and return a queued response (if any)
        self.vendor_reads.push((request, value, index));
        self.receive(timeout_ms)
    }

    fn in_endpoint(&self) -> Option<u8> {
        None
    }

    fn out_endpoint(&self) -> Option<u8> {
        None
    }

    fn clear_halt(&mut self, _endpoint: u8) -> Result<()> {
        // No-op for mock transport
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DeviceType;

    #[test]
    fn mock_transport_basic() {
        let mut m = MockTransport::new(DeviceType::S310);
        m.push_response(vec![0x01]);
        m.send(&[0xaa]).unwrap();
        assert_eq!(m.sent.len(), 1);
        let r = m.receive(1000).unwrap();
        assert_eq!(r, vec![0x01]);
    }

    #[test]
    fn mock_transport_multiple_responses() {
        let mut m = MockTransport::new(DeviceType::S320);
        m.push_response(vec![0x01]);
        m.push_response(vec![0x02]);

        let r1 = m.receive(1000).unwrap();
        assert_eq!(r1, vec![0x01]);
        let r2 = m.receive(1000).unwrap();
        assert_eq!(r2, vec![0x02]);
        // No more responses -> Timeout
        assert!(matches!(m.receive(1000), Err(crate::Error::Timeout)));
    }
}
