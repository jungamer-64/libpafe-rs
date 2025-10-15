// libpafe-rs/libpafe/src/transport/usb/mod.rs

#![cfg(feature = "usb")]

use std::time::Duration;

use crate::transport::traits::Transport;
use crate::types::DeviceType;
use crate::{Error, Result};

use rusb::UsbContext;
use rusb::{Context, DeviceHandle, GlobalContext};

mod descriptor;
use descriptor::find_endpoints;

/// Minimal UsbTransport implementation. This is intentionally small — it
/// detects the first PaSoRi device (Sony vendor id 0x054c) and exposes
/// basic bulk/interrupt send/receive paths. It is feature-gated behind
/// `--features usb` and requires the `rusb` crate.
pub struct UsbTransport {
    handle: DeviceHandle<Context>,
    device_type: DeviceType,
    in_ep: Option<u8>,
    out_ep: Option<u8>,
    timeout_ms: u64,
}

impl UsbTransport {
    /// Open the first matching Sony PaSoRi device found on the bus.
    pub fn open() -> Result<Self> {
        let ctx = Context::new()?;
        for device in ctx.devices()?.iter() {
            let dd = device.device_descriptor()?;
            if dd.vendor_id() == 0x054c {
                if let Some(dt) = DeviceType::from_product_id(dd.product_id()) {
                    let mut handle = device.open()?;

                    // If a kernel driver is attached for interface 0, detach it
                    // so we can claim the interface and perform control/bulk
                    // transfers. This is a common requirement on Linux where
                    // the kernel HID driver may own PaSoRi devices.
                    if let Ok(true) = handle.kernel_driver_active(0) {
                        // Best-effort detach; ignore error if it fails and
                        // let claim_interface report a hard failure.
                        let _ = handle.detach_kernel_driver(0);
                    }

                    // Now claim interface 0. Return error on failure so callers
                    // can decide how to proceed (tests treat DeviceNotFound
                    // specially, other USB errors are propagated).
                    handle.claim_interface(0)?;

                    let (in_ep, out_ep, iface_opt) = find_endpoints(&device);

                    // If we discovered an interface for the endpoints prefer
                    // to detach/claim it. Otherwise fall back to interface 0.
                    let iface = iface_opt.unwrap_or(0);

                    // Ensure interface is claimed for subsequent transfers.
                    if let Ok(true) = handle.kernel_driver_active(iface) {
                        let _ = handle.detach_kernel_driver(iface);
                    }
                    handle.claim_interface(iface)?;

                    return Ok(UsbTransport {
                        handle,
                        device_type: dt,
                        in_ep,
                        out_ep,
                        timeout_ms: 1000,
                    });
                }
            }
        }

        Err(Error::DeviceNotFound)
    }
}

impl Transport for UsbTransport {
    fn send(&mut self, data: &[u8]) -> Result<()> {
        let timeout = Duration::from_millis(self.timeout_ms);

        if let Some(ep) = self.out_ep {
            // Prefer bulk transfer. If that fails try interrupt.
            match self.handle.write_bulk(ep, data, timeout) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    // Try interrupt
                    let _ = self.handle.write_interrupt(ep, data, timeout)?;
                    return Ok(());
                }
            }
        }

        // Fallback to a vendor-specific control transfer if no OUT endpoint
        let req_type = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        );
        let _ = self
            .handle
            .write_control(req_type, 0, 0, 0, data, timeout)?;
        Ok(())
    }

    fn receive(&mut self, timeout_ms: u64) -> Result<Vec<u8>> {
        let timeout = Duration::from_millis(timeout_ms);
        let mut buf = vec![0u8; 512];

        if let Some(ep) = self.in_ep {
            match self.handle.read_bulk(ep, &mut buf, timeout) {
                Ok(n) => {
                    buf.truncate(n);
                    return Ok(buf);
                }
                Err(_) => {
                    let n = self.handle.read_interrupt(ep, &mut buf, timeout)?;
                    buf.truncate(n);
                    return Ok(buf);
                }
            }
        }

        // No IN endpoint — try a control read (rare for PaSoRi but keep a fallback)
        let req_type = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        );
        let n = self
            .handle
            .read_control(req_type, 0, 0, 0, &mut buf, timeout)?;
        buf.truncate(n);
        Ok(buf)
    }

    fn device_type(&self) -> Result<DeviceType> {
        Ok(self.device_type)
    }

    fn reset(&mut self) -> Result<()> {
        // Try to perform a soft reset via vendor-specific control transfer if
        // available. For now just clear endpoints buffer and return Ok.
        Ok(())
    }

    fn control_write(&mut self, data: &[u8]) -> Result<()> {
        let timeout = Duration::from_millis(self.timeout_ms);
        let req_type = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        );
        let _ = self
            .handle
            .write_control(req_type, 0, 0, 0, data, timeout)?;
        Ok(())
    }

    fn vendor_control_write(
        &mut self,
        request: u8,
        value: u16,
        index: u16,
        data: &[u8],
    ) -> Result<()> {
        let timeout = Duration::from_millis(self.timeout_ms);
        let req_type = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        );
        let _ = self
            .handle
            .write_control(req_type, request, value, index, data, timeout)?;
        Ok(())
    }

    fn control_read(&mut self, timeout_ms: u64) -> Result<Vec<u8>> {
        let timeout = Duration::from_millis(timeout_ms);
        let mut buf = vec![0u8; 512];
        let req_type = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        );
        let n = self
            .handle
            .read_control(req_type, 0, 0, 0, &mut buf, timeout)?;
        buf.truncate(n);
        Ok(buf)
    }

    fn vendor_control_read(
        &mut self,
        request: u8,
        value: u16,
        index: u16,
        timeout_ms: u64,
    ) -> Result<Vec<u8>> {
        let timeout = Duration::from_millis(timeout_ms);
        let mut buf = vec![0u8; 512];
        let req_type = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        );
        let n = self
            .handle
            .read_control(req_type, request, value, index, &mut buf, timeout)?;
        buf.truncate(n);
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests require actual hardware and are ignored by default. They
    // are provided as integration points for manual/hardware runners.
    #[test]
    #[ignore]
    fn open_device_if_present() {
        let r = UsbTransport::open();
        match r {
            Ok(mut t) => {
                let dt = t.device_type().unwrap();
                assert!(matches!(
                    dt,
                    crate::types::DeviceType::S310
                        | crate::types::DeviceType::S320
                        | crate::types::DeviceType::S330
                ));
            }
            Err(e) => {
                // If device not found that's acceptable in CI environments
                assert!(matches!(e, crate::Error::DeviceNotFound));
            }
        }
    }
}
