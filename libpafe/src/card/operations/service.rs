use crate::device::Device;
use crate::protocol::{Command, Response};
use crate::types::{Idm, SystemCode};
use crate::{Error, Result};

const REQUEST_SERVICE_RSP: u8 = 0x03;
const REQUEST_RESPONSE_RSP: u8 = 0x05;
const REQUEST_SYSTEM_RSP: u8 = 0x0D;

/// Iterator over service codes returned by SearchServiceCode.
pub struct ServiceIterator<'a> {
    card: &'a crate::card::Card,
    device: &'a mut Device<crate::device::Initialized>,
    current_index: u16,
    finished: bool,
}

impl<'a> ServiceIterator<'a> {
    pub fn new(
        card: &'a crate::card::Card,
        device: &'a mut Device<crate::device::Initialized>,
    ) -> Self {
        Self {
            card,
            device,
            current_index: 0,
            finished: false,
        }
    }
}

/// Request the key versions for the provided service/node codes.
pub fn request_service_versions(
    card: &crate::card::Card,
    device: &mut Device<crate::device::Initialized>,
    node_codes: &[u16],
) -> Result<Vec<u16>> {
    let idm = require_felica(card)?;
    let cmd = Command::RequestService {
        idm,
        node_codes: node_codes.to_vec(),
    };

    match device.execute(cmd, 1000)? {
        Response::RequestService {
            idm: resp_idm,
            versions,
        } if resp_idm == idm => Ok(versions),
        Response::RequestService { .. } => Err(Error::UnexpectedResponse {
            expected: REQUEST_SERVICE_RSP,
            actual: REQUEST_SERVICE_RSP,
        }),
        other => Err(Error::UnexpectedResponse {
            expected: REQUEST_SERVICE_RSP,
            actual: other.response_code(),
        }),
    }
}

/// Query the current operating mode via RequestResponse.
pub fn request_response_mode(
    card: &crate::card::Card,
    device: &mut Device<crate::device::Initialized>,
) -> Result<u8> {
    let idm = require_felica(card)?;
    let cmd = Command::RequestResponse { idm };

    match device.execute(cmd, 1000)? {
        Response::RequestResponse {
            idm: resp_idm,
            mode,
        } if resp_idm == idm => Ok(mode),
        Response::RequestResponse { .. } => Err(Error::UnexpectedResponse {
            expected: REQUEST_RESPONSE_RSP,
            actual: REQUEST_RESPONSE_RSP,
        }),
        other => Err(Error::UnexpectedResponse {
            expected: REQUEST_RESPONSE_RSP,
            actual: other.response_code(),
        }),
    }
}

/// Retrieve the list of published system codes for the card.
pub fn request_system_codes(
    card: &crate::card::Card,
    device: &mut Device<crate::device::Initialized>,
) -> Result<Vec<SystemCode>> {
    let idm = require_felica(card)?;
    let cmd = Command::RequestSystemCode { idm };

    match device.execute(cmd, 1000)? {
        Response::RequestSystemCode {
            idm: resp_idm,
            system_codes,
        } if resp_idm == idm => Ok(system_codes),
        Response::RequestSystemCode { .. } => Err(Error::UnexpectedResponse {
            expected: REQUEST_SYSTEM_RSP,
            actual: REQUEST_SYSTEM_RSP,
        }),
        other => Err(Error::UnexpectedResponse {
            expected: REQUEST_SYSTEM_RSP,
            actual: other.response_code(),
        }),
    }
}

fn require_felica(card: &crate::card::Card) -> Result<Idm> {
    card.idm().copied().ok_or_else(|| {
        Error::UnsupportedOperation("operation is only supported for FeliCa (Type F) cards".into())
    })
}

impl<'a> Iterator for ServiceIterator<'a> {
    type Item = Result<u16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let idm = match self.card.idm() {
            Some(idm) => *idm,
            None => {
                self.finished = true;
                return Some(Err(crate::Error::UnsupportedOperation(
                    "Card does not have IDm (not a FeliCa card)".into(),
                )));
            }
        };

        let cmd = crate::protocol::Command::SearchServiceCode {
            idm,
            index: self.current_index,
        };

        match self.device.execute(cmd, 1000) {
            Ok(crate::protocol::Response::SearchServiceCode {
                area_or_service_code: Some(code),
                ..
            }) => {
                // move to next index and yield code
                self.current_index = self.current_index.saturating_add(1);
                Some(Ok(code))
            }
            Ok(crate::protocol::Response::SearchServiceCode {
                area_or_service_code: None,
                ..
            }) => {
                // termination
                self.finished = true;
                None
            }
            Ok(_) => {
                // unexpected response
                self.finished = true;
                Some(Err(crate::Error::UnexpectedResponse {
                    expected: 0x0b,
                    actual: 0x00,
                }))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}
