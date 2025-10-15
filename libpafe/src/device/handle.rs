// libpafe-rs/libpafe/src/device/handle.rs

use std::marker::PhantomData;

use crate::protocol::codec;
use crate::protocol::{Command, Response};
use crate::transport::Transport;
use crate::types::{DeviceType, Idm, Pmm, SystemCode};
use crate::{Error, Result};

/// Type-state markers
pub struct Uninitialized;
pub struct Initialized;

/// Device handle that enforces initialization state at compile time.
pub struct Device<State = Uninitialized> {
    transport: Box<dyn Transport>,
    device_type: DeviceType,
    model: Box<dyn crate::device::models::DeviceModel>,
    _state: PhantomData<State>,
}

impl Device<Uninitialized> {
    /// Create a Device from an existing Transport instance. This is
    /// primarily intended for tests where a MockTransport is provided.
    pub fn new_with_transport(mut transport: Box<dyn Transport>) -> Result<Self> {
        let device_type = transport.device_type()?;
        let model = crate::device::models::create_model_for(device_type);
        Ok(Self {
            transport,
            device_type,
            model,
            _state: PhantomData,
        })
    }

    /// Initialize the device (transport-level reset and device-specific init
    /// sequences). Returns an initialized Device on success.
    pub fn initialize(self) -> Result<Device<Initialized>> {
        // Basic transport reset and device-model driven initialization.
        let mut this = self;
        this.transport.reset()?;

        // Perform model-specific initialization (S310/S320/S330 etc.)
        // Use the cached model instance stored on the Device.
        this.model.initialize(&mut *this.transport)?;

        Ok(Device {
            transport: this.transport,
            device_type: this.device_type,
            model: this.model,
            _state: PhantomData,
        })
    }

    /// Inspect the detected device type even before initialization.
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
}

impl Device<Initialized> {
    /// Execute a command and return the parsed Response.
    pub fn execute(&mut self, cmd: Command, timeout_ms: u64) -> Result<Response> {
        // Prepare both the raw command payload and the fully-framed
        // FeliCa frame. Device models may choose which form they want to
        // send using the wrap_command hook.
        let payload = cmd.encode();
        let framed = codec::encode_command_frame(&cmd)?;

        // Let the device model wrap the outgoing bytes (PN533 envelopes
        // for S330, vendor-control envelopes for others, etc.).
        // Use the cached model instance to wrap/unwrap model-specific
        // transport payloads.
        let to_send = self.model.wrap_command(&framed, &payload);

        self.transport.send(&to_send)?;

        let raw_resp = self.transport.receive(timeout_ms)?;

        // Allow the model to extract the inner FeliCa frame or payload
        // from a device-specific response format.
        let inner = self.model.unwrap_response(cmd.command_code(), &raw_resp)?;

        let response = codec::decode_response_frame(cmd.command_code(), &inner)?;
        Ok(response)
    }

    /// High-level polling convenience method.
    pub fn polling(&mut self, system_code: SystemCode) -> Result<crate::card::Card> {
        let cmd = Command::Polling {
            system_code,
            request_code: 0,
            time_slot: 0,
        };
        let resp = self.execute(cmd, 1000)?;

        match resp {
            Response::Polling {
                idm,
                pmm,
                system_code,
            } => Ok(crate::card::Card::new(idm, pmm, system_code)),
            _ => Err(Error::PollingFailed),
        }
    }

    /// Model-specific multi-target polling (if supported by the device
    /// model). This delegates to the DeviceModel implementation which may
    /// perform vendor_control transfers and extract multiple embedded
    /// FeliCa frames.
    pub fn list_passive_targets(
        &mut self,
        system_code: SystemCode,
        max_targets: u8,
        timeout_ms: u64,
    ) -> Result<Vec<crate::card::Card>> {
        self.model
            .list_passive_targets(&mut *self.transport, system_code, max_targets, timeout_ms)
    }

    /// Accessor for device type
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Command;
    use crate::transport::mock::MockTransport;
    use crate::types::DeviceType;
    use crate::types::SystemCode;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn mock_device_polling() {
        // Prepare a mock transport with a pre-seeded polling response frame
        let mut mock = MockTransport::new(DeviceType::S320);

        let mut payload = vec![0x01]; // response code for polling
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]); // pmm
        payload.extend_from_slice(&crate::types::SystemCode::new(0x0a0b).to_le_bytes());

        let frame = crate::protocol::Frame::encode(&payload).unwrap();
        // Model initialization will first consume one response; push a
        // dummy ack so the polling payload remains for the polling call.
        mock.push_response(vec![0xAA]);
        mock.push_response(frame);

        let boxed: Box<dyn Transport> = Box::new(mock);
        let device = Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = dev.polling(crate::types::SystemCode::new(0x0a0b)).unwrap();
        assert_eq!(card.idm().as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(card.pmm().as_bytes(), &[9, 10, 11, 12, 13, 14, 15, 16]);
        assert_eq!(card.system_code().as_u16(), 0x0a0b);
    }

    #[test]
    fn device_execute_sends_framed_command() {
        // Shared MockTransport so test can inspect sent messages after Device owns it
        let inner = Rc::new(RefCell::new(MockTransport::new(DeviceType::S320)));

        // Prepare a polling response frame for the transport to return
        let mut payload = vec![0x01];
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]); // pmm
        payload.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());
        let frame = crate::protocol::Frame::encode(&payload).unwrap();
        // The model initialization will consume one response (handshake),
        // so push a dummy init ack first and then the polling frame for
        // the actual execute() call.
        inner.borrow_mut().push_response(vec![0xAA]);
        inner.borrow_mut().push_response(frame);

        // Transport wrapper that delegates into Rc<RefCell<MockTransport>>
        struct SharedTransport {
            inner: Rc<RefCell<MockTransport>>,
        }
        impl SharedTransport {
            fn new(inner: Rc<RefCell<MockTransport>>) -> Self {
                Self { inner }
            }
        }
        impl crate::transport::traits::Transport for SharedTransport {
            fn send(&mut self, data: &[u8]) -> Result<()> {
                self.inner.borrow_mut().send(data)
            }
            fn receive(&mut self, timeout_ms: u64) -> Result<Vec<u8>> {
                self.inner.borrow_mut().receive(timeout_ms)
            }
            fn device_type(&self) -> Result<DeviceType> {
                self.inner.borrow().device_type()
            }
            fn reset(&mut self) -> Result<()> {
                self.inner.borrow_mut().reset()
            }
        }

        let boxed: Box<dyn crate::transport::Transport> =
            Box::new(SharedTransport::new(inner.clone()));
        let device = Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let cmd = Command::Polling {
            system_code: SystemCode::new(0x0a0b),
            request_code: 1,
            time_slot: 0,
        };
        let _ = dev.execute(cmd.clone(), 1000).unwrap();

        // There should be at least two sends: model init and the actual command frame
        let sent = &inner.borrow().sent;
        assert!(
            sent.len() >= 2,
            "expected at least two sends, got {}",
            sent.len()
        );

        let expected_frame = crate::protocol::codec::encode_command_frame(&cmd).unwrap();
        assert_eq!(sent.last().unwrap(), &expected_frame);
    }

    #[test]
    fn mock_device_polling_s330() {
        // Mock transport that will return a PN532-wrapped response
        let mut mock = MockTransport::new(DeviceType::S330);

        // Initialization handshake ack
        mock.push_response(vec![0xAA]);

        // Prepare a FeliCa polling response payload and embed it in a
        // simple PN532 InListPassiveTarget response wrapper so that the
        // S330 model can extract the embedded FeliCa frame.
        let mut payload = vec![0x01]; // response code for polling
        payload.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // idm
        payload.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]); // pmm
        payload.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());

        let frame = crate::protocol::Frame::encode(&payload).unwrap();
        let mut pn_resp = vec![0xD5, 0x4B, 0x01];
        pn_resp.extend_from_slice(&frame);
        mock.push_response(pn_resp);

        let boxed: Box<dyn Transport> = Box::new(mock);
        let device = Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let card = dev.polling(SystemCode::new(0x0a0b)).unwrap();
        assert_eq!(card.idm().as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(card.pmm().as_bytes(), &[9, 10, 11, 12, 13, 14, 15, 16]);
        assert_eq!(card.system_code().as_u16(), 0x0a0b);
    }

    #[test]
    fn mock_device_list_passive_targets_s330() {
        use crate::protocol::Frame;
        use crate::types::SystemCode;

        let mut mock = MockTransport::new(DeviceType::S330);
        mock.push_response(vec![0xAA]);

        let mut p1 = vec![0x01];
        p1.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        p1.extend_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        p1.extend_from_slice(&SystemCode::new(0x0a0b).to_le_bytes());
        let f1 = Frame::encode(&p1).unwrap();

        let mut p2 = vec![0x01];
        p2.extend_from_slice(&[21, 22, 23, 24, 25, 26, 27, 28]);
        p2.extend_from_slice(&[29, 30, 31, 32, 33, 34, 35, 36]);
        p2.extend_from_slice(&SystemCode::new(0x1111).to_le_bytes());
        let f2 = Frame::encode(&p2).unwrap();

        let mut pn = vec![0xD5, 0x4B, 0x02];
        pn.extend_from_slice(&f1);
        pn.extend_from_slice(&f2);
        mock.push_response(pn);

        let boxed: Box<dyn Transport> = Box::new(mock);
        let device = Device::new_with_transport(boxed).unwrap();
        let mut dev = device.initialize().unwrap();

        let cards = dev
            .list_passive_targets(SystemCode::new(0x0a0b), 2, 1000)
            .unwrap();
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].idm().as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(cards[1].idm().as_bytes(), &[21, 22, 23, 24, 25, 26, 27, 28]);
    }
}
