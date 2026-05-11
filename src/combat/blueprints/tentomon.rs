use crate::combat::kernel::{BatteryLoopTransition, CombatKernelTransition};
use crate::combat::state::ResolvedAction;
use crate::data::skills_ron::{CustomSignalPayload, SkillCustomSignal};

use super::CustomSignalDispatchError;

pub const OWNER: &str = "tentomon";

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
        "build_static_charge" => {
            let amount = match signal.payload() {
                CustomSignalPayload::Amount { amount } => amount as u16,
                _ => {
                    return Err(CustomSignalDispatchError::MalformedPayload {
                        owner: OWNER.to_string(),
                        signal: "build_static_charge".to_string(),
                        detail: "expected Amount payload".to_string(),
                    });
                }
            };
            Ok(vec![CombatKernelTransition::BatteryLoop(
                BatteryLoopTransition::build_static_charge(amount as u8),
            )])
        }
        "build_circuit_charge" => {
            let amount = match signal.payload() {
                CustomSignalPayload::Amount { amount } => amount as u16,
                _ => {
                    return Err(CustomSignalDispatchError::MalformedPayload {
                        owner: OWNER.to_string(),
                        signal: "build_circuit_charge".to_string(),
                        detail: "expected Amount payload".to_string(),
                    });
                }
            };
            Ok(vec![CombatKernelTransition::BatteryLoop(
                BatteryLoopTransition::build_circuit_charge(amount as u8),
            )])
        }
        "spend_circuit_charge" => {
            let amount = match signal.payload() {
                CustomSignalPayload::Amount { amount } => amount as u16,
                _ => {
                    return Err(CustomSignalDispatchError::MalformedPayload {
                        owner: OWNER.to_string(),
                        signal: "spend_circuit_charge".to_string(),
                        detail: "expected Amount payload".to_string(),
                    });
                }
            };
            Ok(vec![CombatKernelTransition::BatteryLoop(
                BatteryLoopTransition::spend_circuit_charge(amount as u8),
            )])
        }
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: other.to_string(),
        }),
    }
}
