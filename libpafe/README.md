libpafe (Rust)
=================

Pure Rust implementation of the FeliCa protocol and PaSoRi device
support. This crate provides the protocol encoding/decoding, device
model abstractions, transport traits and test utilities used by the
`libpafe` Rust project.

Build
-----

cd into the crate and build with cargo:

```bash
cd libpafe-rs/libpafe
cargo build
```

Legal Notice
------------

This project is a clean-room, independent implementation of the
FeliCa protocol and PaSoRi device interactions. It was developed based
solely on publicly available specifications and device observations and
does not reuse or derive from the original GPL-licensed C `libpafe`
source code.

Implementation references:

- Sony FeliCa Technical Specification (publicly available)
- NXP PN532 / PN533 documentation (publicly available, protocol-compatible with RCS956)

License: Dual-licensed under `MIT OR Apache-2.0`. See `Cargo.toml` and
`NOTICE` for full attribution and licensing information.

If you plan to use this code in a commercial product, consult an
intellectual property attorney to confirm compliance with third-party
licensing obligations.
