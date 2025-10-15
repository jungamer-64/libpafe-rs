// RC-S310 model module (split per design)
mod commands;
mod config;

use crate::Result;

pub struct S310Model;

impl S310Model {
    pub fn new() -> Self {
        Self
    }
}

impl crate::device::models::DeviceModel for S310Model {
    fn initialize(&self, transport: &mut dyn crate::transport::Transport) -> Result<()> {
        // Conservative S310 init using values from config.
        for attempt in 0..config::ATTEMPTS {
            // Use vendor_control_write/read to express explicit USB request
            // semantics; transports that don't support explicit parameters
            // will fall back to control_write/control_read via trait defaults.
            let _ = transport.vendor_control_write(
                config::INIT_REQUEST,
                config::INIT_VALUE,
                config::INIT_INDEX,
                config::INIT_CMD,
            )?;

            match transport.vendor_control_read(
                config::INIT_REQUEST,
                config::INIT_VALUE,
                config::INIT_INDEX,
                config::READ_TIMEOUT_MS,
            ) {
                Ok(resp) if !resp.is_empty() => {
                    return Ok(());
                }
                Err(e) => {
                    if attempt + 1 >= config::ATTEMPTS {
                        return Err(e);
                    }
                }
                _ => {
                    // empty response -> retry
                }
            }
        }

        Err(crate::Error::Timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::models::DeviceModel;
    use crate::transport::mock::MockTransport;
    use crate::types::DeviceType;
    #[path = "../../../../tests/common/mod.rs"]
    mod common;

    #[test]
    fn s310_model_init_sends_and_receives() {
        let mut m = MockTransport::new(DeviceType::S310);
        common::seed_init_and_frames(&mut m, vec![vec![0xAB]]);
        let model = S310Model::new();
        model.initialize(&mut m).unwrap();
        assert_eq!(m.sent.len(), 1);
        assert_eq!(m.sent[0], vec![0x54]);
    }

    #[test]
    fn s310_model_init_fails_on_timeout() {
        let mut m = MockTransport::new(DeviceType::S310);
        let model = S310Model::new();
        match model.initialize(&mut m) {
            Err(crate::Error::Timeout) => {}
            other => panic!("expected Timeout, got {:?}", other),
        }
    }

    #[test]
    fn s310_model_uses_vendor_control_parameters() {
        let mut mock = MockTransport::new(DeviceType::S310);
        common::seed_init_and_frames(&mut mock, vec![vec![0xAB]]);

        let model = S310Model::new();
        model.initialize(&mut mock).unwrap();

        assert!(mock.vendor_calls.len() >= 1, "expected vendor calls >= 1");
        let (req, val, idx, data) = &mock.vendor_calls[0];
        assert_eq!(*req, super::config::INIT_REQUEST);
        assert_eq!(*val, super::config::INIT_VALUE);
        assert_eq!(*idx, super::config::INIT_INDEX);
        assert_eq!(data, &super::commands::vendor_init());

        assert!(mock.vendor_reads.len() >= 1, "expected vendor reads >= 1");
        let (rreq, rval, ridx) = &mock.vendor_reads[0];
        assert_eq!(*rreq, super::config::INIT_REQUEST);
        assert_eq!(*rval, super::config::INIT_VALUE);
        assert_eq!(*ridx, super::config::INIT_INDEX);
    }
}
