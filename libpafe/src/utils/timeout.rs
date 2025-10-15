//! Timeout helpers used across the crate.
//!
//! Keep these helpers minimal: they centralize the commonly used default
//! timeout value and provide a small conversion helper so tests and code can
//! express timeouts in milliseconds clearly.

use std::time::Duration;

/// Default read timeout in milliseconds used by transports when a caller
/// doesn't provide an explicit timeout.
pub const DEFAULT_READ_TIMEOUT_MS: u64 = 1000;

/// Convert milliseconds to Duration.
pub fn ms(ms: u64) -> Duration {
    Duration::from_millis(ms)
}

/// Convenience: default read timeout as Duration.
pub fn default_read_timeout() -> Duration {
    ms(DEFAULT_READ_TIMEOUT_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ms_to_duration() {
        assert_eq!(ms(500).as_millis(), 500);
    }

    #[test]
    fn default_timeout_positive() {
        assert!(default_read_timeout() >= ms(1));
    }
}
