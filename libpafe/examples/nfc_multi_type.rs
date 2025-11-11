// Multi-type NFC card detection example for RC-S330

// This example demonstrates detecting FeliCa (Type F), Type A, and Type B cards
// using the RC-S330 device's RCS956 chip capabilities.

use libpafe::prelude::*;
use libpafe::transport::usb::UsbTransport;

fn main() -> Result<()> {
    println!("Opening RC-S330 device...");

    // Open a USB transport and build a Device from it. The UsbTransport
    // implementation is feature-gated behind `--features usb` so ensure the
    // example is run with that feature enabled.
    let transport = UsbTransport::open()?;
    let device = Device::new_with_transport(Box::new(transport))?;
    let mut dev = device.initialize()?;

    println!("\n=== Scanning for FeliCa (Type F) cards ===");
    match dev.list_passive_targets(CardType::TypeF, SystemCode::ANY, 3, 1000) {
        Ok(cards) if !cards.is_empty() => {
            println!("Found {} FeliCa card(s):", cards.len());
            for (i, card) in cards.iter().enumerate() {
                if let Some(idm) = card.idm() {
                    println!("  Card {}: IDm = {}", i + 1, idm.to_hex());
                    if let Some(sc) = card.system_code() {
                        println!("           SystemCode = {:04X}", sc.as_u16());
                    }
                }
            }
        }
        Ok(_) => println!("No FeliCa cards detected"),
        Err(e) => println!("FeliCa scan error: {:?}", e),
    }

    println!("\n=== Scanning for Type B cards ===");
    match dev.list_passive_targets(CardType::TypeB, SystemCode::ANY, 3, 1000) {
        Ok(cards) if !cards.is_empty() => {
            println!("Found {} Type B card(s):", cards.len());
            for (i, card) in cards.iter().enumerate() {
                if let Some(uid) = card.uid() {
                    println!("  Card {}: UID = {}", i + 1, uid.to_hex());
                }
            }
        }
        Ok(_) => println!("No Type B cards detected"),
        Err(e) => println!("Type B scan error: {:?}", e),
    }

    println!("\n=== Scanning for Type A cards ===");
    match dev.list_passive_targets(CardType::TypeA, SystemCode::ANY, 3, 1000) {
        Ok(cards) if !cards.is_empty() => {
            println!("Found {} Type A card(s):", cards.len());
            for (i, card) in cards.iter().enumerate() {
                if let Some(uid) = card.uid() {
                    println!("  Card {}: UID = {}", i + 1, uid.to_hex());
                }
            }
        }
        Ok(_) => println!("No Type A cards detected"),
        Err(e) => println!("Type A scan error: {:?}", e),
    }

    println!("\nScan complete.");
    Ok(())
}
