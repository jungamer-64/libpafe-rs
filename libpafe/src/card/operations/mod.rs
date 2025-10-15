pub mod read;
pub mod service;
pub mod write;

// Re-export commonly used functions/types at the operations root so callers
// can use `crate::card::operations::read_blocks(...)` and receive the
// iterator type as `crate::card::operations::ServiceIterator`.
pub use read::{read_blocks, read_single};
pub use service::ServiceIterator;
