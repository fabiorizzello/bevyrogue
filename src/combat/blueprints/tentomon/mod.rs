use crate::combat::bevy_types::*;

use crate::combat::runtime::{SignalPayload, intent::CastId};
use crate::combat::runtime::registry::{ValidationField, ValidationSection};
use crate::combat::kernel::{
    CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition, TacticalCycleTransition,
};
use crate::combat::modifiers::{DamageModifierLedger, ModifierLayer};
use crate::combat::rng::CombatRng;
use crate::combat::types::UnitId;

use super::{CustomSignalDispatchError, amount_payload};

pub mod apply;
pub mod identity;

// Public blueprint surface: kept stable so `tests/` imports via
// `bevyrogue::combat::blueprints::tentomon::{...}` continue to resolve.
pub use apply::{
    apply_battery_loop_transition, apply_battery_loop_transitions_system,
    format_battery_loop_snapshot,
};
pub use identity::{
    BatteryLoopBlockedReason, BatteryLoopChargeKind, BatteryLoopSignal, BatteryLoopSnapshot,
    BatteryLoopState, BatteryLoopStep, BatteryLoopTransition,
};

pub const OWNER: &str = "tentomon";
pub const SIG_BUILD_STATIC_CHARGE: &str = "build_static_charge";
pub const SIG_BUILD_CIRCUIT_CHARGE: &str = "build_circuit_charge";
pub const SIG_SPEND_CIRCUIT_CHARGE: &str = "spend_circuit_charge";
pub const SIG_CYCLE_RESET: &str = "cycle_reset";
const BLOCK_REACTION_CHANCE_PCT: i32 = 30;
const BLOCK_REACTION_MITIGATION_PCT: i32 = 50;

pub const STATIC_CHARGE_THRESHOLD: u8 = 3;
pub const CIRCUIT_CHARGE_CAP: u8 = 3;
pub const BATTERY_ENERGY_GRANT: u8 = 5;

pub struct BatteryLoopHook;

impl CombatKernelHook for BatteryLoopHook {
    fn domain(&self) -> CombatKernelHookDomain {
        CombatKernelHookDomain::Shared
    }

    fn on_transition(
        &self,
        transition: &CombatKernelTransition,
        out: &mut Vec<CombatKernelTransition>,
    ) {
        if matches!(
            transition,
            CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
                wrapped_cycle: true,
                ..
            })
        ) {
            out.push(CombatKernelTransition::Blueprint {
                owner: OWNER.to_string(),
                name: SIG_CYCLE_RESET.to_string(),
                payload: SignalPayload::Amount(0),
            });
        }
    }
}

pub struct TentomonPlugin;

impl Plugin for TentomonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatteryLoopState>().add_systems(
            Update,
            apply_battery_loop_transitions_system,
        );

        app.world_mut()
            .resource_mut::<crate::combat::kernel::CombatKernelRegistry>()
            .register(BatteryLoopHook);

        app.world_mut()
            .resource_mut::<crate::combat::runtime::ExtRegistries>()
            .pre_damage_reactions
            .register("tentomon/block_reaction", resolve_block_reaction_in_world);
    }
}

pub fn register_tentomon_ext(regs: &mut crate::combat::runtime::ExtRegistries) {
    regs.validation
        .register("battery/validation", battery_validation_section);
    regs.pre_damage_reactions
        .register("tentomon/block_reaction", resolve_block_reaction_in_world);
}

fn battery_validation_section(world: &World) -> Option<ValidationSection> {
    world.get_resource::<BatteryLoopState>().map(|state| {
        let snapshot = BatteryLoopSnapshot::from(state);
        ValidationSection::new(
            "battery",
            vec![
                ValidationField::new("static", format!("{}/{}", snapshot.static_charge, snapshot.static_charge_cap)),
                ValidationField::new("circuit", format!("{}/{}", snapshot.circuit_charge, snapshot.circuit_charge_cap)),
                ValidationField::new("block_ready", snapshot.block_reaction_armed.to_string()),
                ValidationField::new(
                    "last",
                    snapshot
                        .last_transition
                        .map(apply::format_battery_loop_transition)
                        .unwrap_or_else(|| "none".to_string()),
                ),
            ],
        )
    })
}

fn blueprint_transition(name: &'static str, amount: i64) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_string(),
        name: name.to_string(),
        payload: SignalPayload::Amount(amount),
    }
}

pub fn dispatch(
    signal: &crate::data::skills_ron::SkillCustomSignal,
    _action: &crate::combat::state::ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_owned(),
        });
    }

    match signal.signal() {
        SIG_BUILD_STATIC_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_BUILD_STATIC_CHARGE)?;
            Ok(vec![blueprint_transition(
                SIG_BUILD_STATIC_CHARGE,
                amount as i64,
            )])
        }
        SIG_BUILD_CIRCUIT_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_BUILD_CIRCUIT_CHARGE)?;
            Ok(vec![blueprint_transition(
                SIG_BUILD_CIRCUIT_CHARGE,
                amount as i64,
            )])
        }
        SIG_SPEND_CIRCUIT_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_SPEND_CIRCUIT_CHARGE)?;
            Ok(vec![blueprint_transition(
                SIG_SPEND_CIRCUIT_CHARGE,
                amount as i64,
            )])
        }
        SIG_CYCLE_RESET => Ok(vec![blueprint_transition(SIG_CYCLE_RESET, 0)]),
        _ => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_owned(),
            signal: signal.signal().to_owned(),
        }),
    }
}

pub fn resolve_block_reaction_in_world(
    world: &mut World,
    target: UnitId,
    cast_id: CastId,
) -> Option<i32> {
    if !world
        .get_resource::<BatteryLoopState>()?
        .block_reaction_ready()
    {
        return None;
    }

    let rolled = world.resource_scope(|_w, mut rng: Mut<CombatRng>| {
        rng.roll_pct(BLOCK_REACTION_CHANCE_PCT)
    });
    if !rolled {
        return None;
    }

    world.resource_scope(|_w, mut state: Mut<BatteryLoopState>| {
        state.proc_block_reaction();
        state.last_block_reaction_cast_id = Some(cast_id);
    });

    if let Some(mut ledger) = world.get_resource_mut::<DamageModifierLedger>() {
        ledger.arm(target, ModifierLayer::Passive, BLOCK_REACTION_MITIGATION_PCT);
    }

    debug!(
        "Tentomon block reaction triggered for target={:?} cast_id={:?}",
        target, cast_id
    );

    Some(BLOCK_REACTION_MITIGATION_PCT)
}
