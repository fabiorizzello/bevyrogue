use crate::combat::kernel::{
    CombatKernelTransition, PrecisionCommitment, PrecisionMindGameTransition, PrecisionOutcome,
    PrecisionReveal, PrecisionWindowKind,
};
use crate::combat::state::ResolvedAction;
use crate::data::skills_ron::SkillCustomSignal;

use super::CustomSignalDispatchError;

pub const OWNER: &str = "renamon";

pub fn dispatch(
    signal: &SkillCustomSignal,
    _action: &ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_owned(),
        });
    }

    match signal.signal() {
        "open_momentum_window" => Ok(vec![CombatKernelTransition::PrecisionMindGame(
            PrecisionMindGameTransition::open_window(PrecisionWindowKind::Momentum),
        )]),
        "commit_precision_press" => Ok(vec![CombatKernelTransition::PrecisionMindGame(
            PrecisionMindGameTransition::commit(PrecisionCommitment::Press),
        )]),
        "reveal_bait" => Ok(vec![CombatKernelTransition::PrecisionMindGame(
            PrecisionMindGameTransition::reveal(PrecisionReveal::Baited),
        )]),
        "resolve_precision_success" => Ok(vec![CombatKernelTransition::PrecisionMindGame(
            PrecisionMindGameTransition::resolve(PrecisionOutcome::Success),
        )]),
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: other.to_string(),
        }),
    }
}
