use std::convert::TryFrom;

use crate::combat::kernel::{CombatKernelTransition, HolySupportTransition};
use crate::combat::state::ResolvedAction;
use crate::data::skills_ron::{CustomSignalPayload, SkillCustomSignal};

use super::CustomSignalDispatchError;

pub const OWNER: &str = "patamon";

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
        "build_holy_support_grace" => {
            let amount = match signal.payload() {
                CustomSignalPayload::Amount { amount } => u8::try_from(amount).map_err(|_| {
                    CustomSignalDispatchError::MalformedPayload {
                        owner: OWNER.to_owned(),
                        signal: signal.signal().to_owned(),
                        detail: format!("amount {amount} does not fit in u8"),
                    }
                })?,
                payload => {
                    return Err(CustomSignalDispatchError::MalformedPayload {
                        owner: OWNER.to_owned(),
                        signal: signal.signal().to_owned(),
                        detail: format!("expected Amount payload, found {payload:?}"),
                    });
                }
            };

            if amount == 0 {
                return Err(CustomSignalDispatchError::MalformedPayload {
                    owner: OWNER.to_owned(),
                    signal: signal.signal().to_owned(),
                    detail: "holy support grace amount must be positive".to_owned(),
                });
            }

            Ok(vec![CombatKernelTransition::HolySupport(
                HolySupportTransition::build_grace(amount),
            )])
        }
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_owned(),
            signal: other.to_owned(),
        }),
    }
}
