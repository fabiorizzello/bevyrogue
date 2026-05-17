use crate::combat::api::SignalPayload;
use crate::combat::kernel::CombatKernelTransition;
use crate::combat::state::ResolvedAction;
use crate::data::skills_ron::{CustomSignalPayload, SkillCustomSignal};

use super::super::CustomSignalDispatchError;

pub const OWNER: &str = "dorumon";
pub const SIGNAL_BUILD_EXPLOIT: &str = "build_exploit";
pub const SIGNAL_APPLY_PREY_LOCK: &str = "apply_prey_lock";
pub const SIGNAL_CONSUME_PREY_LOCK_PAYOFF: &str = "consume_prey_lock_payoff";
pub const SIGNAL_ENTER_BERSERK: &str = "enter_berserk";
pub const SIGNAL_TICK: &str = "tick";

pub(crate) fn blueprint_transition(name: &'static str, amount: i64) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_string(),
        name: name.to_string(),
        payload: SignalPayload::Amount(amount),
    }
}

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
        SIGNAL_BUILD_EXPLOIT => {
            let amount = match signal.payload() {
                CustomSignalPayload::Amount { amount } => amount as i64,
                _ => {
                    return Err(CustomSignalDispatchError::MalformedPayload {
                        owner: OWNER.to_string(),
                        signal: SIGNAL_BUILD_EXPLOIT.to_string(),
                        detail: "expected Amount payload".to_string(),
                    });
                }
            };
            Ok(vec![blueprint_transition(SIGNAL_BUILD_EXPLOIT, amount)])
        }
        SIGNAL_APPLY_PREY_LOCK => Ok(vec![blueprint_transition(SIGNAL_APPLY_PREY_LOCK, 0)]),
        SIGNAL_CONSUME_PREY_LOCK_PAYOFF => Ok(vec![blueprint_transition(
            SIGNAL_CONSUME_PREY_LOCK_PAYOFF,
            1,
        )]),
        SIGNAL_ENTER_BERSERK => Ok(vec![blueprint_transition(SIGNAL_ENTER_BERSERK, 0)]),
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: other.to_string(),
        }),
    }
}
