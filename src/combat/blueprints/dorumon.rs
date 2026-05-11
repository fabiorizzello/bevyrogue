use crate::combat::kernel::{CombatKernelTransition, PredatorLoopTransition};
use crate::combat::state::ResolvedAction;
use crate::data::skills_ron::{CustomSignalPayload, SkillCustomSignal};

use super::CustomSignalDispatchError;

pub const OWNER: &str = "dorumon";

pub fn dispatch(
    signal: &SkillCustomSignal,
    action: &ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_owned(),
        });
    }

    match signal.signal() {
        "build_exploit" => {
            let amount = match signal.payload() {
                CustomSignalPayload::Amount { amount } => amount as u16,
                _ => {
                    return Err(CustomSignalDispatchError::MalformedPayload {
                        owner: OWNER.to_string(),
                        signal: "build_exploit".to_string(),
                        detail: "expected Amount payload".to_string(),
                    });
                }
            };
            Ok(vec![CombatKernelTransition::PredatorLoop(
                PredatorLoopTransition::build_exploit(action.target, amount),
            )])
        }
        "apply_prey_lock" => {
            let amount = match signal.payload() {
                CustomSignalPayload::Amount { amount } => amount as u16,
                _ => 0,
            };
            Ok(vec![CombatKernelTransition::PredatorLoop(
                PredatorLoopTransition::apply_prey_lock(action.target, amount),
            )])
        }
        "consume_prey_lock_payoff" => Ok(vec![CombatKernelTransition::PredatorLoop(
            PredatorLoopTransition::consume_prey_lock_payoff(action.target, 1),
        )]),
        "enter_berserk" => Ok(vec![CombatKernelTransition::PredatorLoop(
            PredatorLoopTransition::enter_berserk(0),
        )]),
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: other.to_string(),
        }),
    }
}
