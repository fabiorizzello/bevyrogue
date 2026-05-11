use crate::combat::types::UnitId;
use crate::data::party_ron::PartyConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartyConfigError {
    WrongTamer { got: UnitId },
}

impl std::fmt::Display for PartyConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartyConfigError::WrongTamer { got } => {
                write!(f, "tamer_id {:?} is not Taichi (UnitId(0))", got)
            }
        }
    }
}

impl std::error::Error for PartyConfigError {}

pub fn validate_party_config(party: &PartyConfig) -> Result<(), PartyConfigError> {
    if party.tamer_id != UnitId(0) {
        return Err(PartyConfigError::WrongTamer {
            got: party.tamer_id,
        });
    }
    Ok(())
}
