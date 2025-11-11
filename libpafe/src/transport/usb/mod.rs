// libpafe-rs/libpafe/src/transport/usb/mod.rs

#![cfg(feature = "usb")]

use std::time::Duration;

use crate::protocol::Frame;
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
            // Attempt several attempts to write to the OUT endpoint
            // (bulk preferred, interrupt fallback). On repeated
            // failures attempt to clear any halt/stall on the endpoint
            // and back off between retries.
            let mut last_rusb: Option<rusb::Error> = None;
            for attempt in 1..=3 {
                // If caller provided a PN532 host payload (TFI=0xD4), try
                // to send a framed host packet first; fall back to raw.
                if !data.is_empty() && data[0] == crate::constants::PN532_CMD_PREFIX_HOST {
                    if let Ok(framed) = Frame::encode(data) {
                        match self.handle.write_bulk(ep, &framed, timeout) {
                            Ok(_) => return Ok(()),
                            Err(e) => {
                                last_rusb = Some(e);
                                // Try interrupt as a fallback
                                match self.handle.write_interrupt(ep, &framed, timeout) {
                                    Ok(_) => return Ok(()),
                                    Err(e2) => {
                                        last_rusb = Some(e2);
                                        let _ = self.handle.clear_halt(ep);
                                        std::thread::sleep(Duration::from_millis(
                                            20 * attempt as u64,
                                        ));
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }

                match self.handle.write_bulk(ep, data, timeout) {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        last_rusb = Some(e);
                        match self.handle.write_interrupt(ep, data, timeout) {
                            Ok(_) => return Ok(()),
                            Err(e2) => {
                                last_rusb = Some(e2);
                                let _ = self.handle.clear_halt(ep);
                                std::thread::sleep(Duration::from_millis(20 * attempt as u64));
                                continue;
                            }
                        }
                    }
                }
            }
            if let Some(e) = last_rusb {
                return Err(e.into());
            }
            return Err(Error::Timeout);
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
            // Attempt multiple attempts to read from the IN endpoint
            // to tolerate transient IO errors or endpoint stalls. On
            // failure try clearing the halt and retry with a small
            // backoff. If an ACK-only PN532 frame is observed, attempt
            // a follow-up read but treat a follow-up failure as
            // non-fatal (return the ACK bytes) so callers can still
            // examine the transport buffer.
            let mut last_err: Option<rusb::Error> = None;
            let pn532_ack = [0x00u8, 0x00u8, 0xFFu8, 0x00u8, 0xFFu8, 0x00u8];
            for attempt in 1..=3 {
                match self.handle.read_bulk(ep, &mut buf, timeout) {
                    Ok(n) => {
                        buf.truncate(n);
                        // If ACK-only, try a short follow-up read and
                        // append bytes if present; on follow-up error,
                        // still return the ACK bytes.
                        if self.device_type == DeviceType::S330 && buf == pn532_ack {
                            let mut follow = vec![0u8; 512];
                            match self.handle.read_bulk(ep, &mut follow, timeout) {
                                Ok(n2) => {
                                    follow.truncate(n2);
                                    buf.extend_from_slice(&follow);
                                }
                                Err(_) => {
                                    // Best-effort: try interrupt endpoint as a
                                    // last-ditch follow-up. Ignore follow-up
                                    // failures as callers may already have
                                    // useful ACK data.
                                    let _ = self.handle.read_interrupt(ep, &mut follow, timeout);
                                }
                            }
                        }
                        return Ok(buf);
                    }
                    Err(e) => {
                        last_err = Some(e);
                        // Try reading via interrupt endpoint as a
                        // fallback before attempting to recover the
                        // bulk endpoint.
                        match self.handle.read_interrupt(ep, &mut buf, timeout) {
                            Ok(n) => {
                                buf.truncate(n);
                                return Ok(buf);
                            }
                            Err(_) => {
                                // Clear a possible halt/stall and retry.
                                let _ = self.handle.clear_halt(ep);
                                std::thread::sleep(Duration::from_millis(20 * attempt as u64));
                                continue;
                            }
                        }
                    }
                }
            }
            // All attempts failed: return the last rusb error if present
            if let Some(e) = last_err {
                return Err(e.into());
            }
            return Err(Error::Timeout);
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

    fn in_endpoint(&self) -> Option<u8> {
        self.in_ep
    }

    fn out_endpoint(&self) -> Option<u8> {
        self.out_ep
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
        // If endpoints are available prefer the endpoint (bulk/interrupt)
        // path over control transfers. This mirrors how many readers
        // expose the PN532 host interface on their bulk endpoints and
        // avoids vendor-control pipe stalls on some systems.
        if self.out_ep.is_some() {
            return self.send(data);
        }

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
        // Prefer the endpoint read path if an IN endpoint is present so
        // that PN532 responses returned on the bulk endpoint are read
        // instead of attempting potentially-stalling control reads.
        if self.in_ep.is_some() {
            return self.receive(timeout_ms);
        }

        let timeout = Duration::from_millis(timeout_ms);
        let mut buf = vec![0u8; 512];
        let req_type = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Vendor,
            rusb::Recipient::Device,
        );
        // Retry control read a few times since some systems may return
        // transient errors for vendor control reads. Backoff slightly
        // between retries.
        let mut last_rusb: Option<rusb::Error> = None;
        for _ in 0..3 {
            match self
                .handle
                .read_control(req_type, request, value, index, &mut buf, timeout)
            {
                Ok(n) => {
                    buf.truncate(n);
                    return Ok(buf);
                }
                Err(e) => {
                    last_rusb = Some(e);
                    std::thread::sleep(Duration::from_millis(30));
                    continue;
                }
            }
        }
        if let Some(e) = last_rusb {
            return Err(e.into());
        }
        Err(Error::Timeout)
    }

    fn clear_halt(&mut self, endpoint: u8) -> Result<()> {
        // Clear a halt/stall on the given endpoint. Propagate rusb errors.
        self.handle.clear_halt(endpoint)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests require actual hardware and are ignored by default. They
    // are provided as integration points for manual/hardware runners.
    #[test]
    #[ignore = "requires hardware (PaSoRi)"]
    fn open_device_if_present() {
        let r = UsbTransport::open();
        match r {
            Ok(t) => {
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
