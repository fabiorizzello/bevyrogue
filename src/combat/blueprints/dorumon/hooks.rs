use bevy::prelude::*;

use crate::combat::{
    api::applier::intent_applier,
    events::{CombatEvent, CombatEventKind},
    kernel::{
        CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition, TacticalCycleTransition,
    },
    types::UnitId,
    unit::Unit,
};

use super::identity::{PredatorLoopHook, PredatorLoopState};
use super::signals::{SIGNAL_TICK, blueprint_transition};

impl CombatKernelHook for PredatorLoopHook {
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
            out.push(blueprint_transition(SIGNAL_TICK, 0));
        }
    }
}

pub fn register_passive_runtime(app: &mut App) {
    app.add_systems(
        Update,
        apply_predator_loop_enemy_kill_system.after(intent_applier),
    );
}

fn apply_predator_loop_enemy_kill_system(
    mut events: MessageReader<CombatEvent>,
    units: Query<&Unit>,
    mut state: ResMut<PredatorLoopState>,
) {
    for event in events.read() {
        let CombatEventKind::OnEnemyKill = &event.kind else {
            continue;
        };

        let source_is_dorumon = units
            .iter()
            .any(|unit| unit.id == event.source && unit.name == "Dorumon");
        if !source_is_dorumon {
            continue;
        }

        state.track_target(event.target);
        let _ = state.build_exploit(event.target, 1);
        let _ = state.apply_prey_lock(event.target);
    }
}
