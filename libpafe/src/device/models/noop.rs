// libpafe-rs/libpafe/src/device/models/noop.rs

use crate::Result;

pub struct NoopModel;

impl NoopModel {
    pub fn new() -> Self {
        Self
    }
}

impl crate::device::models::DeviceModel for NoopModel {
    fn initialize(&self, _transport: &mut dyn crate::transport::Transport) -> Result<()> {
        // No-op initialization for unknown device types.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::models::DeviceModel;
    use crate::transport::mock::MockTransport;
    use crate::types::DeviceType;

    #[test]
    fn noop_model_does_nothing() {
        let mut mock = MockTransport::new(DeviceType::S310);
        let m = NoopModel::new();
        m.initialize(&mut mock).unwrap();

        assert_eq!(mock.sent.len(), 0);
    }
}
