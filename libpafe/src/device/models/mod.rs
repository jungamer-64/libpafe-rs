// libpafe-rs/src/device/models/mod.rs

use crate::Result;
use crate::types::DeviceType;

pub trait DeviceModel {
    /// Initialize the device via the provided transport. Implementations may
    /// send device-specific sequences (control/interrupt/bulk) necessary to
    /// bring the device to an operational state.
    fn initialize(&self, transport: &mut dyn crate::transport::Transport) -> Result<()>;

    /// Wrap a raw FeliCa command payload for the device model. The default
    /// implementation returns the original payload unchanged. S330 will
    /// override this to wrap the payload in a PN533 InListPassiveTarget
    /// packet.
    /// Wrap a command for transport. Implementations receive both the
    /// fully-framed FeliCa frame (`framed`) and the raw protocol payload
    /// (`payload`). By default the framed form is sent unchanged.
    fn wrap_command(&self, framed: &[u8], _payload: &[u8]) -> Vec<u8> {
        framed.to_vec()
    }

    /// Unwrap a raw device response into the inner FeliCa payload that can
    /// be passed to the protocol-level Response decoder. The default
    /// implementation returns the raw bytes unchanged. Device-specific
    /// implementations (e.g. S330) may strip protocol headers and return
    /// only the inner payload.
    fn unwrap_response(&self, _expected_cmd: u8, raw: &[u8]) -> Result<Vec<u8>> {
        Ok(raw.to_vec())
    }

    /// Optional model-specific multi-target polling routine. Some device
    /// implementations (e.g. S330/PN533) provide a vendor-control based
    /// InListPassiveTarget operation that can return multiple targets in a
    /// single response. The default implementation returns an error to
    /// indicate the operation is not supported by the model.
    ///
    /// For FeliCa (Type F), system_code is used. For Type A/B, it's ignored.
    fn list_passive_targets(
        &self,
        _transport: &mut dyn crate::transport::Transport,
        _card_type: crate::types::CardType,
        _system_code: crate::types::SystemCode,
        _max_targets: u8,
        _timeout_ms: u64,
    ) -> Result<Vec<crate::card::Card>> {
        Err(crate::Error::PollingFailed)
    }

    /// Model-specific helper: extract candidate FeliCa wire frames from a
    /// raw device response buffer. Default implementation returns an
    /// empty list which signals no model-specific candidates are
    /// available. Device implementations (e.g. S330) may override this
    /// to provide richer extraction heuristics.
    fn extract_candidate_frames(&self, _raw: &[u8], _expected_cmd: u8) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

mod noop;
// Include the per-device implementations from their directory-style modules.
// We use `include!` to prefer the new `s310/mod.rs` etc. even if legacy
// `s310.rs` files exist in the tree. This lets us migrate to the directory
// layout incrementally without deleting legacy files in the same commit.
pub mod s310 {
    include!("s310/mod.rs");
}
pub use s310::S310Model;

pub mod s320 {
    include!("s320/mod.rs");
}
pub use s320::S320Model;

pub mod s330 {
    include!("s330/mod.rs");
}
pub use s330::S330Model;

/// Factory to create a model implementation for a DeviceType. Unknown
/// devices receive a no-op model.
pub fn create_model_for(device_type: DeviceType) -> Box<dyn DeviceModel> {
    match device_type {
        DeviceType::S310 => Box::new(s310::S310Model::new()),
        DeviceType::S320 => Box::new(S320Model::new()),
        DeviceType::S330 => Box::new(s330::S330Model::new()),
        _ => Box::new(noop::NoopModel::new()),
    }
}
