use bevy::prelude::*;

use crate::combat::{
    api::{applier::intent_applier, intent::CastId, SignalPayload},
    events::{CombatEvent, CombatEventKind},
    kernel::CombatKernelTransition,
    modifiers::{DamageModifierLedger, ModifierLayer},
    rng::CombatRng,
    types::UnitId,
    unit::Unit,
};

use super::{CustomSignalDispatchError, amount_payload};
use crate::combat::battery_loop::BatteryLoopState;

pub const OWNER: &str = "tentomon";
pub const SIG_BUILD_STATIC_CHARGE: &str = "build_static_charge";
pub const SIG_BUILD_CIRCUIT_CHARGE: &str = "build_circuit_charge";
pub const SIG_SPEND_CIRCUIT_CHARGE: &str = "spend_circuit_charge";
pub const SIG_CYCLE_RESET: &str = "cycle_reset";
const BLOCK_REACTION_CHANCE_PCT: i32 = 30;
const BLOCK_REACTION_MITIGATION_PCT: i32 = 50;

pub struct TentomonPlugin;

impl Plugin for TentomonPlugin {
    fn build(&self, app: &mut App) {
        register_passive_runtime(app);
    }
}

pub fn register_passive_runtime(app: &mut App) {
    app.add_systems(
        Update,
        apply_tentomon_block_reaction_system.after(intent_applier),
    );
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
            Ok(vec![blueprint_transition(SIG_BUILD_STATIC_CHARGE, amount as i64)])
        }
        SIG_BUILD_CIRCUIT_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_BUILD_CIRCUIT_CHARGE)?;
            Ok(vec![blueprint_transition(SIG_BUILD_CIRCUIT_CHARGE, amount as i64)])
        }
        SIG_SPEND_CIRCUIT_CHARGE => {
            let amount = amount_payload(signal, OWNER, SIG_SPEND_CIRCUIT_CHARGE)?;
            Ok(vec![blueprint_transition(SIG_SPEND_CIRCUIT_CHARGE, amount as i64)])
        }
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: other.to_string(),
        }),
    }
}

/// Run Tentomon's reactive block loop against an incoming-damage combat event.
///
/// Returns `true` when the passive actually armed the mitigation ledger.
pub fn resolve_block_reaction_for_event(
    target: UnitId,
    cast_id: CastId,
    target_is_tentomon: bool,
    state: &mut BatteryLoopState,
    ledger: &mut DamageModifierLedger,
    rng: &mut CombatRng,
) -> bool {
    if state.last_block_reaction_cast_id == Some(cast_id) {
        return false;
    }

    if !target_is_tentomon || !state.block_reaction_ready() {
        return false;
    }

    state.last_block_reaction_cast_id = Some(cast_id);

    if !rng.roll_pct(BLOCK_REACTION_CHANCE_PCT) {
        return false;
    }

    ledger.arm(
        target,
        ModifierLayer::Passive,
        BLOCK_REACTION_MITIGATION_PCT,
    );
    let _ = state.proc_block_reaction();
    true
}

/// World-facing wrapper used by the damage pipeline so Tentomon can react before HP mutation.
pub fn resolve_block_reaction_in_world(
    world: &mut World,
    target: UnitId,
    cast_id: CastId,
    target_is_tentomon: bool,
) -> bool {
    let mut outcome = false;
    world.resource_scope(|world, mut state: Mut<BatteryLoopState>| {
        world.resource_scope(|world, mut ledger: Mut<DamageModifierLedger>| {
            world.resource_scope(|_world, mut rng: Mut<CombatRng>| {
                outcome = resolve_block_reaction_for_event(
                    target,
                    cast_id,
                    target_is_tentomon,
                    &mut state,
                    &mut ledger,
                    &mut rng,
                );
            });
        });
    });
    outcome
}

fn apply_tentomon_block_reaction_system(
    mut events: MessageReader<CombatEvent>,
    units: Query<&Unit>,
    mut state: ResMut<BatteryLoopState>,
    mut ledger: ResMut<DamageModifierLedger>,
    mut rng: ResMut<CombatRng>,
) {
    for event in events.read() {
        let CombatEventKind::IncomingDamage { .. } = &event.kind else {
            continue;
        };

        let target_is_tentomon = units.iter().any(|unit| {
            unit.id == event.target && unit.name == "Tentomon"
        });

        let _ = resolve_block_reaction_for_event(
            event.target,
            event.cast_id,
            target_is_tentomon,
            &mut state,
            &mut ledger,
            &mut rng,
        );
    }
}
