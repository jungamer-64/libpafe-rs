// libpafe-rs/libpafe/src/lib.rs

//! libpafe
//!
//! Pure Rust implementation for Sony PaSoRi NFC readers.
#![warn(missing_docs)]

pub mod card;
pub mod constants;
pub mod device;
pub mod error;
pub mod prelude;
pub mod protocol;
pub mod test_support;
pub mod transport;
pub mod types;
pub mod utils;

// Re-export common types at crate root so `crate::Error`, `crate::Result`,
// and the newtypes in `types` are available for consumers and for
// convenient `prelude` re-exports.
pub use crate::error::*;
pub use crate::types::*;

pub use prelude::*;
