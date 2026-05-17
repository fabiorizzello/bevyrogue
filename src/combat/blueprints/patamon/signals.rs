use std::convert::TryFrom;

use crate::combat::api::SignalPayload;
use crate::combat::kernel::CombatKernelTransition;
use crate::combat::state::ResolvedAction;
use crate::data::skills_ron::{CustomSignalPayload, SkillCustomSignal};

use super::{SIGNAL_BUILD_HOLY_SUPPORT_GRACE, SIGNAL_CYCLE_RESET, SIGNAL_MARK_MARTYR_LIGHT,
    SIGNAL_SPEND_HOLY_SUPPORT_GRACE};
use super::super::CustomSignalDispatchError;

pub const OWNER: &str = "patamon";

fn blueprint_transition(name: &str, payload: SignalPayload) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_owned(),
        name: name.to_owned(),
        payload,
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
        SIGNAL_BUILD_HOLY_SUPPORT_GRACE => {
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

            Ok(vec![blueprint_transition(
                SIGNAL_BUILD_HOLY_SUPPORT_GRACE,
                SignalPayload::Amount(i64::from(amount)),
            )])
        }
        SIGNAL_SPEND_HOLY_SUPPORT_GRACE => {
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

            Ok(vec![blueprint_transition(
                SIGNAL_SPEND_HOLY_SUPPORT_GRACE,
                SignalPayload::Amount(i64::from(amount)),
            )])
        }
        SIGNAL_MARK_MARTYR_LIGHT => Ok(vec![blueprint_transition(
            SIGNAL_MARK_MARTYR_LIGHT,
            SignalPayload::Empty,
        )]),
        SIGNAL_CYCLE_RESET => Ok(vec![blueprint_transition(
            SIGNAL_CYCLE_RESET,
            SignalPayload::Empty,
        )]),
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_owned(),
            signal: other.to_owned(),
        }),
    }
}
