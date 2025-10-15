// libpafe-rs/libpafe/src/device/builder.rs

use crate::device::handle::{Device, Uninitialized};
use crate::transport::Transport;
use crate::{Error, Result};

/// Helper to construct a Device with optional configuration.
pub struct DeviceBuilder {
    transport: Option<Box<dyn Transport>>,
}

impl DeviceBuilder {
    pub fn new() -> Self {
        Self { transport: None }
    }

    /// Provide an already-created transport instance (e.g. MockTransport)
    pub fn with_transport(mut self, transport: Box<dyn Transport>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Consume the builder and return an uninitialized Device.
    /// Requires a transport to be provided; otherwise returns DeviceNotFound.
    pub fn build_uninitialized(self) -> Result<Device<Uninitialized>> {
        match self.transport {
            Some(t) => Device::new_with_transport(t),
            None => Err(Error::DeviceNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::MockTransport;
    use crate::types::DeviceType;

    #[test]
    fn builder_with_mock_transport() {
        let mock = MockTransport::new(DeviceType::S320);
        let boxed: Box<dyn Transport> = Box::new(mock);
        let device = DeviceBuilder::new()
            .with_transport(boxed)
            .build_uninitialized()
            .unwrap();
        assert_eq!(device.device_type(), DeviceType::S320);
    }
}
