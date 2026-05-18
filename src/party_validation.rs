use crate::combat::types::UnitId;
use crate::data::party_ron::PartyConfig;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PartyConfigError {
    #[error("tamer_id {got:?} is not Taichi (UnitId(0))")]
    WrongTamer { got: UnitId },
}

pub fn validate_party_config(party: &PartyConfig) -> Result<(), PartyConfigError> {
    if party.tamer_id != UnitId(0) {
        return Err(PartyConfigError::WrongTamer {
            got: party.tamer_id,
        });
    }
    Ok(())
}
