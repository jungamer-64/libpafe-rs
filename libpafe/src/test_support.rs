//! Test support helpers intended for use by unit and integration tests.
//!
//! These helpers centralize common MockTransport setup so tests across the
//! crate and tests/ directory can reuse the same logic.
#![allow(dead_code)]

use crate::{device, transport, types, Result};

/// Build a MockTransport pre-seeded with the given framed responses and
/// return it boxed as a Transport trait object.
#[doc(hidden)]
pub fn boxed_mock_with_responses(
    device_type: types::DeviceType,
    responses: Vec<Vec<u8>>,
) -> Box<dyn transport::traits::Transport> {
    let mut mock = transport::mock::MockTransport::new(device_type);
    for resp in responses {
        mock.push_response(resp);
    }
    Box::new(mock)
}

/// Convenience: create and initialize a Device<Initialized> backed by a
/// MockTransport pre-seeded with the provided responses. The caller may
/// pass the exact frames the device expects (init handshake first).
#[doc(hidden)]
pub fn initialized_mock_device(
    device_type: types::DeviceType,
    responses: Vec<Vec<u8>>,
) -> Result<device::Device<device::Initialized>> {
    let boxed = boxed_mock_with_responses(device_type, responses);
    let device = device::Device::new_with_transport(boxed)?;
    let initialized = device.initialize()?;
    Ok(initialized)
}

/// Push a handshake ack (0xAA) then additional frames onto a MockTransport.
/// Frames are pushed as-is; they may be framed FeliCa frames or raw bytes
/// depending on the test's needs.
#[doc(hidden)]
pub fn seed_init_and_frames(mock: &mut transport::mock::MockTransport, frames: Vec<Vec<u8>>) {
    // Default init ack used by device models in tests
    mock.push_response(vec![0xAA]);
    for f in frames {
        mock.push_response(f);
    }
}
