use crate::types::{Idm, Pmm, SystemCode};

/// Compact information describing a FeliCa card (IDm/PMm/SystemCode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardInfo {
    pub idm: Idm,
    pub pmm: Pmm,
    pub system_code: SystemCode,
}

impl CardInfo {
    pub fn new(idm: Idm, pmm: Pmm, system_code: SystemCode) -> Self {
        Self {
            idm,
            pmm,
            system_code,
        }
    }

    pub fn idm(&self) -> &Idm {
        &self.idm
    }

    pub fn pmm(&self) -> &Pmm {
        &self.pmm
    }

    pub fn system_code(&self) -> SystemCode {
        self.system_code
    }
}

impl From<&crate::card::Card> for CardInfo {
    fn from(card: &crate::card::Card) -> Self {
        // CardInfo only supports FeliCa (Type F) cards
        match card {
            crate::card::Card::TypeF {
                idm,
                pmm,
                system_code,
            } => CardInfo::new(*idm, *pmm, *system_code),
            _ => panic!("CardInfo can only be created from Type F (FeliCa) cards"),
        }
    }
}
