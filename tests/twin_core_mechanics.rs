use bevy::prelude::*;

use bevyrogue::combat::blueprints::twin_core::{
    TwinCoreDesignTag, TwinCoreHook, TwinCoreSignal, TwinCoreState,
    apply_twin_core_transitions_system, twin_core_design_tag,
};
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kernel::{
    CombatBeatId, CombatKernelRegistry, CombatKernelTransition, CombatTagChangeKind,
    CombatTagState, CombatTagTransition, TacticalCyclePhase, TacticalCycleStep,
    TacticalCycleTransition,
};
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::types::UnitId;

fn route_transition(
    app: &mut App,
    transition: CombatKernelTransition,
    source: UnitId,
    target: UnitId,
) {
    let outputs = {
        let registry = app.world().resource::<CombatKernelRegistry>();
        registry.dispatch(transition)
    };

    for transition in outputs {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source,
            target,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    app.update();
}

fn added_tag(tag: TwinCoreDesignTag) -> CombatKernelTransition {
    let tag = twin_core_design_tag(tag);
    CombatKernelTransition::Tag(CombatTagTransition {
        before: CombatTagState::new(tag.clone(), 3),
        after: CombatTagState::new(tag, 3),
        kind: CombatTagChangeKind::Added,
    })
}

fn consumed_tag(tag: TwinCoreDesignTag) -> CombatKernelTransition {
    let tag = twin_core_design_tag(tag);
    let mut after = CombatTagState::new(tag.clone(), 1);
    after.consumed = true;
    CombatKernelTransition::Tag(CombatTagTransition {
        before: CombatTagState::new(tag, 1),
        after,
        kind: CombatTagChangeKind::Consumed,
    })
}

fn cycle_reset() -> CombatKernelTransition {
    CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
        before: TacticalCycleStep {
            phase: TacticalCyclePhase::Applied,
            step_in_phase: 0,
            cycle_index: 0,
        },
        after: TacticalCycleStep {
            phase: TacticalCyclePhase::Declared,
            step_in_phase: 0,
            cycle_index: 1,
        },
        wrapped_phase: true,
        wrapped_cycle: true,
    })
}

#[test]
fn twin_core_kernel_hooks_apply_current_state_fields() {
    let mut app = App::new();

    let mut registry = CombatKernelRegistry::new();
    registry.register(TwinCoreHook);

    app.insert_resource(registry);
    app.init_resource::<TwinCoreState>();
    app.add_message::<CombatEvent>();
    app.add_systems(Update, apply_twin_core_transitions_system);

    let source = UnitId(1);
    let target = UnitId(2);

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::Heated),
        source,
        target,
    );
    {
        let state = app.world().resource::<TwinCoreState>();
        assert_eq!(state.cross_resonance, 1);
        assert!(state.active_thermal_spark_targets.is_empty());
        assert_eq!(
            state
                .last_signal
                .expect("Heated should build resonance")
                .signal,
            TwinCoreSignal::BuildCrossResonance
        );
    }

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::ThermalSpark),
        source,
        target,
    );
    {
        let state = app.world().resource::<TwinCoreState>();
        assert_eq!(state.cross_resonance, 1);
        assert_eq!(state.active_thermal_spark_targets, vec![target]);
        assert_eq!(
            state
                .last_signal
                .expect("Thermal Spark should be recorded")
                .signal,
            TwinCoreSignal::ThermalSpark
        );
    }

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::Chilled),
        source,
        target,
    );
    {
        let state = app.world().resource::<TwinCoreState>();
        assert_eq!(state.cross_resonance, 2);
        assert_eq!(state.active_thermal_spark_targets, vec![target]);
    }

    route_transition(
        &mut app,
        CombatKernelTransition::Beat(CombatBeatId::Damage),
        source,
        target,
    );
    {
        let state = app.world().resource::<TwinCoreState>();
        assert_eq!(state.cross_resonance, 0);
        assert!(state.active_thermal_spark_targets.is_empty());
        assert!(state.twin_burst_used_this_cycle);
        assert_eq!(
            state
                .last_signal
                .expect("Damage beat should trigger Twin Burst")
                .signal,
            TwinCoreSignal::TwinBurst
        );
    }

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::MeltdownCrack),
        source,
        target,
    );
    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::DeepCrack),
        source,
        target,
    );
    {
        let state = app.world().resource::<TwinCoreState>();
        assert_eq!(state.fire_spend_markers, 1);
        assert_eq!(state.ice_spend_markers, 1);
    }

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::ThermalSpark),
        source,
        target,
    );
    route_transition(
        &mut app,
        consumed_tag(TwinCoreDesignTag::ThermalSpark),
        source,
        target,
    );
    {
        let state = app.world().resource::<TwinCoreState>();
        assert!(state.shatter_used_this_cycle);
        assert!(state.active_thermal_spark_targets.is_empty());
        assert_eq!(
            state
                .last_signal
                .expect("Consumed spark should trigger Shatter")
                .signal,
            TwinCoreSignal::Shatter
        );
    }

    route_transition(&mut app, cycle_reset(), source, target);
    {
        let state = app.world().resource::<TwinCoreState>();
        assert_eq!(state.fire_spend_markers, 0);
        assert_eq!(state.ice_spend_markers, 0);
        assert!(!state.twin_burst_used_this_cycle);
        assert!(!state.shatter_used_this_cycle);
        assert_eq!(
            state
                .last_signal
                .expect("Cycle wrap should reset guards")
                .signal,
            TwinCoreSignal::CycleReset
        );
    }
}
