use crate::combat::kernel::CombatKernelTransition;
use crate::combat::state::ResolvedAction;
use crate::data::skills_ron::SkillCustomSignal;

use super::{TwinCoreDesignTag, twin_core_added_tag_transition};
use super::super::{CustomSignalDispatchError, amount_payload};

pub const OWNER: &str = "agumon";
const APPLY_HEATED: &str = "apply_heated";
const APPLY_MELTDOWN_CRACK: &str = "apply_meltdown_crack";
const APPLY_THERMAL_SPARK: &str = "apply_thermal_spark";

enum AgumonSignal {
    ApplyHeated { turns_left: u16 },
    ApplyMeltdownCrack { turns_left: u16 },
    ApplyThermalSpark { turns_left: u16 },
}

fn parse(signal: &SkillCustomSignal) -> Result<AgumonSignal, CustomSignalDispatchError> {
    match signal.signal() {
        APPLY_HEATED => {
            let amount = amount_payload(signal, OWNER, APPLY_HEATED)?;
            Ok(AgumonSignal::ApplyHeated { turns_left: amount })
        }
        APPLY_MELTDOWN_CRACK => {
            let amount = amount_payload(signal, OWNER, APPLY_MELTDOWN_CRACK)?;
            Ok(AgumonSignal::ApplyMeltdownCrack { turns_left: amount })
        }
        APPLY_THERMAL_SPARK => {
            let amount = amount_payload(signal, OWNER, APPLY_THERMAL_SPARK)?;
            Ok(AgumonSignal::ApplyThermalSpark { turns_left: amount })
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
        AgumonSignal::ApplyHeated { turns_left } => {
            twin_core_added_tag_transition(TwinCoreDesignTag::Heated, turns_left as u8)
        }
        AgumonSignal::ApplyMeltdownCrack { turns_left } => {
            twin_core_added_tag_transition(TwinCoreDesignTag::MeltdownCrack, turns_left as u8)
        }
        AgumonSignal::ApplyThermalSpark { turns_left } => {
            twin_core_added_tag_transition(TwinCoreDesignTag::ThermalSpark, turns_left as u8)
        }
    };

    Ok(vec![transition])
}
