use crate::combat::kernel::CombatKernelTransition;
use crate::combat::state::ResolvedAction;
use crate::combat::blueprints::agumon::{TwinCoreDesignTag, twin_core_added_tag_transition};
use crate::data::skills_ron::SkillCustomSignal;

use super::{CustomSignalDispatchError, amount_payload};

pub const OWNER: &str = "gabumon";
const APPLY_CHILLED: &str = "apply_chilled";
const APPLY_DEEP_CRACK: &str = "apply_deep_crack";
const APPLY_THERMAL_SPARK: &str = "apply_thermal_spark";

enum GabumonSignal {
    ApplyChilled { turns_left: u16 },
    ApplyDeepCrack { turns_left: u16 },
    ApplyThermalSpark { turns_left: u16 },
}

fn parse(signal: &SkillCustomSignal) -> Result<GabumonSignal, CustomSignalDispatchError> {
    match signal.signal() {
        APPLY_CHILLED => {
            let amount = amount_payload(signal, OWNER, APPLY_CHILLED)?;
            Ok(GabumonSignal::ApplyChilled { turns_left: amount })
        }
        APPLY_DEEP_CRACK => {
            let amount = amount_payload(signal, OWNER, APPLY_DEEP_CRACK)?;
            Ok(GabumonSignal::ApplyDeepCrack { turns_left: amount })
        }
        APPLY_THERMAL_SPARK => {
            let amount = amount_payload(signal, OWNER, APPLY_THERMAL_SPARK)?;
            Ok(GabumonSignal::ApplyThermalSpark { turns_left: amount })
        }
        signal => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: signal.to_string(),
        }),
    }
}

pub fn dispatch(
    signal: &SkillCustomSignal,
    _action: &ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_string(),
        });
    }

    let transition = match parse(signal)? {
        GabumonSignal::ApplyChilled { turns_left } => {
            twin_core_added_tag_transition(TwinCoreDesignTag::Chilled, turns_left as u8)
        }
        GabumonSignal::ApplyDeepCrack { turns_left } => {
            twin_core_added_tag_transition(TwinCoreDesignTag::DeepCrack, turns_left as u8)
        }
        GabumonSignal::ApplyThermalSpark { turns_left } => {
            twin_core_added_tag_transition(TwinCoreDesignTag::ThermalSpark, turns_left as u8)
        }
    };

    Ok(vec![transition])
}
