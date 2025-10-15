# Hardware tests

Hardware tests require a physical PaSoRi device connected to the host and
are marked `#[ignore]` so they are not executed on CI by default.

How to run a single hardware test manually (example for S320):

```bash
# Build with usb feature enabled then run ignored tests
cargo test -p libpafe --features usb --test s320_test -- --ignored
```

Prerequisites:

- libusb (platform package)
- Appropriate udev rules on Linux to access USB device
- The test runner must run with permissions to access the device (udev or root)

Each test in this directory attempts to open a transport with
`UsbTransport::open()` and will succeed gracefully if no device is present.
