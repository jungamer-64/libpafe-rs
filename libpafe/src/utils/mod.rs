//! Utilities for libpafe: small, reusable helpers used across the crate.
//!
//! This module intentionally contains tiny, well-tested helpers that are
//! convenient for debug printing (hex) and timeout manipulation.

pub mod hex;
pub mod timeout;

// Re-export the most common helpers at the `utils` module level so callers can
// use `crate::utils::bytes_to_hex(...)` etc if they prefer.
pub use hex::*;
pub use timeout::*;
