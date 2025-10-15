#![allow(dead_code)]

use crate::types::{Idm, Pmm, SystemCode};

/// Minimal builder placeholder for `Card` construction.
///
/// The full builder is planned for a later iteration; this stub keeps the
/// public API surface expected by the migration plan.
pub struct CardBuilder {
    idm: Option<Idm>,
    pmm: Option<Pmm>,
    system_code: Option<SystemCode>,
}

impl CardBuilder {
    pub fn new() -> Self {
        Self {
            idm: None,
            pmm: None,
            system_code: None,
        }
    }

    pub fn idm(mut self, idm: Idm) -> Self {
        self.idm = Some(idm);
        self
    }

    pub fn pmm(mut self, pmm: Pmm) -> Self {
        self.pmm = Some(pmm);
        self
    }

    pub fn system_code(mut self, sc: SystemCode) -> Self {
        self.system_code = Some(sc);
        self
    }

    pub fn build(self) -> crate::Result<crate::card::Card> {
        // Simple, clear behavior for MVP: require all fields.
        let idm = self.idm.ok_or(crate::Error::InvalidLength {
            expected: 8,
            actual: 0,
        })?;
        let pmm = self.pmm.ok_or(crate::Error::InvalidLength {
            expected: 8,
            actual: 0,
        })?;
        let system_code = self.system_code.ok_or(crate::Error::InvalidLength {
            expected: 2,
            actual: 0,
        })?;

        Ok(crate::card::Card::new(idm, pmm, system_code))
    }
}
