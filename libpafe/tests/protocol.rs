// Aggregator for protocol integration tests located in `tests/protocol/`.
// Cargo treats each top-level file in `tests/` as an integration test crate;
// we include the per-topic files as submodules to keep the directory layout
// neat while still allowing `cargo test` to discover them.

#[path = "protocol/frame_integration_test.rs"]
mod frame_integration_test;

#[path = "protocol/checksum_test.rs"]
mod checksum_test;

#[path = "protocol/command_encode_test.rs"]
mod command_encode_test;

#[path = "protocol/response_decode_test.rs"]
mod response_decode_test;
