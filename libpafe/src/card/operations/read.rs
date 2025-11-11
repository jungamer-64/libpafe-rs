use crate::device::Device;
use crate::protocol::{Command, Response};
use crate::types::{BlockData, BlockElement, ServiceCode};
use crate::{Error, Result};

/// Read multiple blocks from a card using ReadWithoutEncryption.
pub fn read_blocks(
    card: &crate::card::Card,
    device: &mut Device<crate::device::Initialized>,
    services: &[ServiceCode],
    blocks: &[BlockElement],
) -> Result<Vec<BlockData>> {
    let idm = card.idm().ok_or_else(|| {
        Error::UnsupportedOperation("Card does not have IDm (not a FeliCa card)".into())
    })?;

    let cmd = Command::ReadWithoutEncryption {
        idm: *idm,
        services: services.to_vec(),
        blocks: blocks.to_vec(),
    };

    let resp = device.execute(cmd, 1000)?;

    match resp {
        Response::ReadWithoutEncryption { blocks, .. } => Ok(blocks),
        _ => Err(Error::PollingFailed),
    }
}

/// Convenience helper that reads a single block.
pub fn read_single(
    card: &crate::card::Card,
    device: &mut Device<crate::device::Initialized>,
    service: ServiceCode,
    block: u16,
) -> Result<BlockData> {
    let blocks = read_blocks(
        card,
        device,
        &[service],
        &[BlockElement::new(
            0,
            crate::types::AccessMode::DirectAccessOrRead,
            block,
        )],
    )?;

    blocks.into_iter().next().ok_or(Error::PollingFailed)
}
