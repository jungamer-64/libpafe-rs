mod commands;
mod config;

use crate::Result;

pub struct S320Model;

impl S320Model {
    pub fn new() -> Self {
        Self
    }
}

impl crate::device::models::DeviceModel for S320Model {
    fn initialize(&self, transport: &mut dyn crate::transport::Transport) -> Result<()> {
        let mut ok = false;
        for attempt in 0..config::INIT_ATTEMPTS {
            // Prefer vendor-specific control transfer with explicit
            // request/value/index for S320 initialisation. Transports that
            // do not support explicit parameters will fall back to
            // `control_write`/`control_read` via the Transport trait
            // default implementations.
            let _ = transport.vendor_control_write(
                config::INIT1_REQUEST,
                config::INIT1_VALUE,
                config::INIT1_INDEX,
                commands::init1(),
            )?;

            match transport.vendor_control_read(
                config::INIT1_REQUEST,
                config::INIT1_VALUE,
                config::INIT1_INDEX,
                config::READ_TIMEOUT_MS,
            ) {
                Ok(resp) if !resp.is_empty() => {
                    ok = true;
                    break;
                }
                _ => {
                    // try interrupt/bulk receive as a fallback
                    match transport.receive(config::READ_TIMEOUT_MS) {
                        Ok(resp2) if !resp2.is_empty() => {
                            ok = true;
                            break;
                        }
                        Err(e2) => {
                            if attempt + 1 >= config::INIT_ATTEMPTS {
                                return Err(e2);
                            }
                        }
                        _ => {
                            // continue retrying
                        }
                    }
                }
            }
        }

        if !ok {
            return Err(crate::Error::Timeout);
        }

        // Finalize initialization using explicit vendor control write.
        let _ = transport.vendor_control_write(
            config::INIT2_REQUEST,
            config::INIT2_VALUE,
            config::INIT2_INDEX,
            commands::init2(),
        )?;
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
    fn s320_model_init_sends_sequence() {
        let mut mock = MockTransport::new(DeviceType::S320);
        mock.push_response(vec![0xAA]);
        let model = S320Model::new();
        model.initialize(&mut mock).unwrap();
        assert_eq!(mock.sent.len(), 2);
        assert_eq!(mock.sent[0], vec![0x5C, 0x01]);
        assert_eq!(mock.sent[1], vec![0x5C, 0x02]);
    }

    #[test]
    fn s320_model_uses_vendor_control_parameters() {
        let mut mock = MockTransport::new(DeviceType::S320);
        mock.push_response(vec![0xAA]);

        let model = S320Model::new();
        model.initialize(&mut mock).unwrap();

        // Ensure vendor_control_write was invoked for the init sequence with
        // the configured request/value/index and that the payload matches
        // the command bytes.
        assert!(mock.vendor_calls.len() >= 2, "expected vendor calls >= 2");
        let (req1, val1, idx1, data1) = &mock.vendor_calls[0];
        assert_eq!(*req1, super::config::INIT1_REQUEST);
        assert_eq!(*val1, super::config::INIT1_VALUE);
        assert_eq!(*idx1, super::config::INIT1_INDEX);
        assert_eq!(data1, &commands::init1());

        let (req2, val2, idx2, data2) = &mock.vendor_calls[1];
        assert_eq!(*req2, super::config::INIT2_REQUEST);
        assert_eq!(*val2, super::config::INIT2_VALUE);
        assert_eq!(*idx2, super::config::INIT2_INDEX);
        assert_eq!(data2, &commands::init2());
    }

    #[test]
    fn s320_model_init_retries_and_fails_on_timeout() {
        let mut mock = MockTransport::new(DeviceType::S320);
        let model = S320Model::new();
        match model.initialize(&mut mock) {
            Err(crate::Error::Timeout) => {}
            other => panic!("expected Timeout, got {:?}", other),
        }
    }

    #[test]
    fn s320_model_fallback_to_receive_on_control_fail() {
        let mut mock = MockTransport::new(DeviceType::S320);
        mock.set_control_failures(1);
        mock.push_response(vec![0xBB]);

        let model = S320Model::new();
        model.initialize(&mut mock).unwrap();

        assert_eq!(mock.sent.len(), 2);
        assert_eq!(mock.sent[0], vec![0x5C, 0x01]);
        assert_eq!(mock.sent[1], vec![0x5C, 0x02]);
    }
}
