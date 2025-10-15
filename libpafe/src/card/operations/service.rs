use crate::device::Device;
use crate::{Error, Result};

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

impl<'a> Iterator for ServiceIterator<'a> {
    type Item = Result<u16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let cmd = crate::protocol::Command::SearchServiceCode {
            idm: self.card.idm,
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
