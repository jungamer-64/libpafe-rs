// libpafe-rs/libpafe/src/transport/usb/descriptor.rs

#![allow(dead_code)]

#[cfg(feature = "usb")]
use rusb::{Device, Direction};

/// Inspect the device descriptors and return the first IN and OUT endpoint
/// addresses found on the device (if any).
#[cfg(feature = "usb")]
/// Returns (in_endpoint, out_endpoint, interface_number)
pub fn find_endpoints<D: rusb::UsbContext>(
    device: &Device<D>,
) -> (Option<u8>, Option<u8>, Option<u8>) {
    if let Ok(config) = device.config_descriptor(0) {
        let mut in_ep = None;
        let mut out_ep = None;
        let mut iface = None;

        for interface in config.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    let addr = endpoint_desc.address();
                    if endpoint_desc.direction() == Direction::In && in_ep.is_none() {
                        in_ep = Some(addr);
                        iface = Some(interface_desc.interface_number());
                    } else if endpoint_desc.direction() == Direction::Out && out_ep.is_none() {
                        out_ep = Some(addr);
                        iface = Some(interface_desc.interface_number());
                    }
                }
            }
        }

        return (in_ep, out_ep, iface);
    }

    (None, None, None)
}
