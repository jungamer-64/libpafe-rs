#![cfg(feature = "usb")]

//! Simple probe example for RC-S330 (RCS956) devices.
//!
//! Usage:
//!   cargo run -p libpafe --example s330_probe --features usb --release

use libpafe::transport::traits::Transport;
use libpafe::{Error, Result, device, transport, types, utils};

fn main() -> Result<()> {
    match transport::usb::UsbTransport::open() {
        Ok(mut t) => {
            println!("Opened PaSoRi device: {:?}", t.device_type()?);
            println!(
                "Transport endpoints: IN={:?} OUT={:?}",
                t.in_endpoint(),
                t.out_endpoint()
            );

            // Send a RCS956 GetVersion command (raw D4 02) and print the raw reply
            let rcs956_get_version: [u8; 2] = [0xD4, 0x02];
            println!("Sending RCS956 GetVersion: {:02x?}", rcs956_get_version);
            let _ = t.vendor_control_write(0x00, 0x0000, 0x0000, &rcs956_get_version);
            match t.vendor_control_read(0x00, 0x0000, 0x0000, 500) {
                Ok(ver) => println!("RCS956 GetVersion raw: {}", utils::bytes_to_hex(&ver)),
                Err(e) => println!("RCS956 GetVersion read failed (non-fatal): {:?}", e),
            }

            // Turn RF on (best-effort) so the reader will attempt to
            // discover targets. Ignore errors from the optional read.
            let rcs956_rf_on: [u8; 4] = [0xD4, 0x32, 0x01, 0x01];
            let _ = t.vendor_control_write(0x00, 0x0000, 0x0000, &rcs956_rf_on);
            let _ = t.vendor_control_read(0x00, 0x0000, 0x0000, 200);

            // Perform a manual InListPassiveTarget send/receive using the
            // transport's bulk/interrupt path (preferred when endpoints are
            // available). This helps diagnose whether the RCS956 InList
            // command reaches the device and if any raw bytes are returned.
            {
                use libpafe::protocol::commands::polling as polling_cmd;
                let payload = polling_cmd::encode_polling(types::SystemCode::ANY, 0, 0);
                let mut inlist = vec![0xD4u8, 0x4A, 0x01u8, 0x01u8];
                inlist.extend_from_slice(&payload);
                println!(
                    "Manual InListPassiveTarget -> {}",
                    utils::bytes_to_hex(&inlist)
                );
                match t.send(&inlist) {
                    Ok(()) => println!("manual send ok"),
                    Err(e) => println!("manual send error: {:?}", e),
                }
                match t.receive(2000) {
                    Ok(resp) => println!("manual receive raw: {}", utils::bytes_to_hex(&resp)),
                    Err(e) => {
                        println!("manual receive error: {:?}", e);
                        // If an IN endpoint is known, try clearing a possible
                        // halt/stall and reattempt the receive once.
                        if let Some(ep) = t.in_endpoint() {
                            println!("attempting clear_halt on endpoint 0x{:02x}", ep);
                            let _ = t.clear_halt(ep);
                            match t.receive(2000) {
                                Ok(resp2) => println!(
                                    "receive after clear_halt: {}",
                                    utils::bytes_to_hex(&resp2)
                                ),
                                Err(e2) => println!("receive after clear_halt failed: {:?}", e2),
                            }
                        }
                    }
                }

                // As an alternative, try sending a fully-framed FeliCa wire
                // frame (preamble + checksums) and see whether the reader
                // replies differently.
                {
                    use libpafe::protocol::Frame;
                    // Build a PN532 host-command payload (TFI=0xD4) + InListPassiveTarget
                    let polling_payload = libpafe::protocol::commands::polling::encode_polling(
                        types::SystemCode::ANY,
                        0,
                        0,
                    );
                    let mut host_payload = vec![
                        libpafe::constants::PN532_CMD_PREFIX_HOST,
                        libpafe::constants::PN532_CMD_INLIST_PASSIVE_TARGET,
                        0x01u8,
                        0x01u8,
                    ];
                    host_payload.extend_from_slice(&polling_payload);

                    if let Ok(framed) = Frame::encode(&host_payload) {
                        println!(
                            "Trying PN532-host-framed payload -> {}",
                            utils::bytes_to_hex(&framed)
                        );
                        let _ = t.send(&framed);
                        // First read should return ACK
                        match t.receive(2000) {
                            Ok(ack) => {
                                println!("framed receive (ACK): {}", utils::bytes_to_hex(&ack))
                            }
                            Err(e3) => println!("framed receive (ACK) error: {:?}", e3),
                        }
                        // Then attempt a second read to get the actual response
                        match t.receive(3000) {
                            Ok(resp4) => println!(
                                "framed receive (response): {}",
                                utils::bytes_to_hex(&resp4)
                            ),
                            Err(e4) => println!("framed receive (response) error: {:?}", e4),
                        }
                    }
                }
            }

            // Move transport into a Device and initialize the reader
            let boxed: Box<dyn transport::traits::Transport> = Box::new(t);
            let device_uninit = device::Device::new_with_transport(boxed)?;
            let mut device = device_uninit.initialize()?;

            // First try a single-target polling using the standard send/receive
            // path (bulk/interrupt endpoints if available). This avoids the
            // vendor-control path which some systems may not support.
            println!("Attempting single-target Polling (Device::polling)...");
            match device.polling(types::SystemCode::ANY) {
                Ok(card) => {
                    let idm_hex = card
                        .idm()
                        .map(|i| i.to_hex())
                        .unwrap_or_else(|| "<none>".to_string());
                    let pmm_hex = card
                        .pmm()
                        .map(|p| utils::bytes_to_hex(p.as_bytes()))
                        .unwrap_or_else(|| "<none>".to_string());
                    let sc_val = card.system_code().map(|sc| sc.as_u16()).unwrap_or(0);
                    println!(
                        "Found card: IDM={} PMM={} SC={:04x}",
                        idm_hex, pmm_hex, sc_val
                    );
                }
                Err(e) => {
                    println!(
                        "Device::polling failed: {:?}. Trying list_passive_targets fallback...",
                        e
                    );
                    match device.list_passive_targets(
                        crate::types::CardType::TypeF,
                        types::SystemCode::ANY,
                        4,
                        2000,
                    ) {
                        Ok(cards) => {
                            if cards.is_empty() {
                                println!("No cards found");
                            } else {
                                for (i, c) in cards.iter().enumerate() {
                                    let idm_hex = c
                                        .idm()
                                        .map(|i| i.to_hex())
                                        .unwrap_or_else(|| "<none>".to_string());
                                    let pmm_hex = c
                                        .pmm()
                                        .map(|p| utils::bytes_to_hex(p.as_bytes()))
                                        .unwrap_or_else(|| "<none>".to_string());
                                    let sc_val = c.system_code().map(|sc| sc.as_u16()).unwrap_or(0);
                                    println!(
                                        "Card #{} IDM={} PMM={} SC={:04x}",
                                        i + 1,
                                        idm_hex,
                                        pmm_hex,
                                        sc_val
                                    );
                                }
                            }
                        }
                        Err(e2) => {
                            println!("list_passive_targets also failed: {:?}", e2);
                        }
                    }
                }
            }

            Ok(())
        }
        Err(Error::DeviceNotFound) => {
            println!("No PaSoRi device found on the USB bus");
            Ok(())
        }
        Err(e) => Err(e),
    }
}
