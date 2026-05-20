//! Twin Core blueprint — kernel hooks, transitions, and signal dispatch.
//!
//! Replaces two single-purpose files (3 tests total):
//!   * `twin_core_mechanics.rs`   (kernel-hook unit-state assertions)
//!   * `twin_core_integration.rs` (validation snapshot + skill→signal dispatch)

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
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::runtime::{CastId, ExtRegistries};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::types::{SkillId, UnitId};
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::skills_ron::{SkillBook, SkillDef};
use bevyrogue::data::units_ron::{UnitDef, UnitRoster};

// ──────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ──────────────────────────────────────────────────────────────────────────────

fn load_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

fn load_skill_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

fn unit_def<'a>(roster: &'a UnitRoster, name: &str) -> &'a UnitDef {
    roster
        .0
        .iter()
        .find(|unit| unit.name == name)
        .unwrap_or_else(|| panic!("missing canonical unit {name}"))
}

fn skill_def<'a>(book: &'a SkillBook, id: &str) -> &'a SkillDef {
    book.0
        .iter()
        .find(|skill| skill.id == SkillId(id.into()))
        .unwrap_or_else(|| panic!("missing canonical skill {id}"))
}

fn runtime_unit(def: &UnitDef) -> Unit {
    Unit {
        id: def.id,
        name: def.name.clone(),
        hp_max: def.hp_max,
        hp_current: def.hp_max,
        attribute: def.attribute,
        resists: def.resists.clone(),
        evo_stage: def.evo_stage,
    }
}

fn spawn_snapshot_unit(app: &mut App, def: &UnitDef) {
    app.world_mut().spawn((
        runtime_unit(def),
        def.team,
        Toughness::new(def.toughness_max, def.weaknesses.clone()),
        UltimateCharge {
            current: 0,
            trigger: def.ultimate_trigger,
            cap: def.ultimate_cap,
            trigger_type: def.ultimate_accumulation_trigger,
            charge_per_event: def.ultimate_charge_per_event,
        },
    ));
}

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

/// Minimal app for kernel-hook tests (no validation snapshots).
fn kernel_app() -> App {
    let mut app = App::new();
    let mut registry = CombatKernelRegistry::new();
    registry.register(TwinCoreHook);

    app.insert_resource(registry);
    app.init_resource::<TwinCoreState>();
    app.add_message::<CombatEvent>();
    app.add_systems(Update, apply_twin_core_transitions_system);
    app
}

/// Heavier app for the validation-snapshot test (CombatState, SP, ActionLog,
/// ExtRegistries with all blueprint validation hooks).
fn snapshot_app() -> App {
    let mut app = App::new();
    let mut registry = CombatKernelRegistry::new();
    registry.register(TwinCoreHook);

    app.insert_resource(registry)
        .init_resource::<TwinCoreState>()
        .init_resource::<ExtRegistries>()
        .insert_resource(CombatState::default())
        .insert_resource(SpPool {
            current: 10,
            max: 10,
        })
        .insert_resource(ActionLog::default())
        .add_message::<CombatEvent>()
        .add_systems(Update, apply_twin_core_transitions_system);

    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        bevyrogue::combat::blueprints::register_all_blueprint_validation_exts(&mut regs);
    }

    app
}

fn capture_snapshot_string(app: &mut App) -> String {
    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should capture");
    format_validation_snapshot(&snapshot)
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn twin_core_kernel_hooks_apply_current_state_fields() {
    let mut app = kernel_app();
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

#[test]
fn canonical_fire_ice_twin_core_loop_is_visible_through_validation_snapshots() {
    let roster = load_roster();
    let skills = load_skill_book();

    let agumon = unit_def(&roster, "Agumon");
    let gabumon = unit_def(&roster, "Gabumon");
    let greymon = unit_def(&roster, "Greymon");
    let garurumon = unit_def(&roster, "Garurumon");
    let ogremon = unit_def(&roster, "Ogremon");

    assert_eq!(skill_def(&skills, "sharp_claws").id, agumon.basic_skill);
    assert_eq!(skill_def(&skills, "agumon_ult").id, agumon.ultimate_skill);
    assert_eq!(skill_def(&skills, "bubble_blast").id, gabumon.basic_skill);
    assert_eq!(skill_def(&skills, "gabumon_ult").id, gabumon.ultimate_skill);
    assert_eq!(skill_def(&skills, "greymon_basic").id, greymon.basic_skill);
    assert_eq!(
        skill_def(&skills, "garurumon_basic").id,
        garurumon.basic_skill
    );

    let mut app = snapshot_app();
    for def in [agumon, gabumon, greymon, garurumon, ogremon] {
        spawn_snapshot_unit(&mut app, def);
    }

    let source = agumon.id;
    let target = ogremon.id;

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::Heated),
        source,
        target,
    );
    let after_fire = capture_snapshot_string(&mut app);
    assert!(after_fire.contains("cr=1"), "{after_fire}");
    assert!(after_fire.contains("spark_targets=[]"), "{after_fire}");
    assert!(after_fire.contains("last=build(1)"), "{after_fire}");

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::Chilled),
        source,
        target,
    );
    let after_ice = capture_snapshot_string(&mut app);
    assert!(after_ice.contains("cr=2"), "{after_ice}");
    assert!(after_ice.contains("spark_targets=[]"), "{after_ice}");
    assert!(after_ice.contains("last=build(1)"), "{after_ice}");

    route_transition(
        &mut app,
        added_tag(TwinCoreDesignTag::ThermalSpark),
        source,
        target,
    );
    let after_spark = capture_snapshot_string(&mut app);
    assert!(
        after_spark.contains(&format!("spark_targets=[{}]", target.0)),
        "{after_spark}"
    );
    assert!(after_spark.contains("last=spark(1)"), "{after_spark}");

    route_transition(
        &mut app,
        CombatKernelTransition::Beat(CombatBeatId::Damage),
        source,
        target,
    );
    let after_beat = capture_snapshot_string(&mut app);
    assert!(after_beat.contains("cr=0"), "{after_beat}");
    assert!(after_beat.contains("spark_targets=[]"), "{after_beat}");
    assert!(after_beat.contains("burst_guard=true"), "{after_beat}");
    assert!(after_beat.contains("last=twin-burst(1)"), "{after_beat}");
    assert!(after_beat.contains("units=["), "{after_beat}");

    let final_snapshot = capture_snapshot_string(&mut app);
    assert!(final_snapshot.contains("cr=0"), "{final_snapshot}");
    assert!(
        final_snapshot.contains("burst_guard=true"),
        "{final_snapshot}"
    );
}

#[test]
fn skill_resolution_emits_twin_core_signals_through_blueprints() {
    let roster = load_roster();
    let skills = load_skill_book();

    let agumon_def = unit_def(&roster, "Agumon");
    let ogremon_def = unit_def(&roster, "Ogremon");

    let mut app = snapshot_app();
    spawn_snapshot_unit(&mut app, agumon_def);
    spawn_snapshot_unit(&mut app, ogremon_def);

    let intent = bevyrogue::combat::turn_system::ActionIntent::Basic {
        attacker: agumon_def.id,
        target: ogremon_def.id,
    };

    let agumon_kit = bevyrogue::combat::kit::UnitSkills {
        basic: agumon_def.basic_skill.clone(),
        skills: vec![],
        ultimate: agumon_def.ultimate_skill.clone(),
        follow_up: None,
    };

    let resolved = bevyrogue::combat::resolution::resolve_action(&intent, &agumon_kit, Some(&skills))
        .expect("should resolve sharp_claws");

    assert_eq!(resolved.skill_id, SkillId("sharp_claws".into()));
    assert!(!resolved.custom_signals.is_empty());
    assert_eq!(resolved.custom_signals[0].owner(), "agumon");
    assert_eq!(resolved.custom_signals[0].signal(), "apply_heated");

    let transitions = bevyrogue::combat::blueprints::transitions_for_action(&resolved);
    assert_eq!(transitions.len(), 1);

    match &transitions[0] {
        CombatKernelTransition::Tag(tag) => {
            assert_eq!(
                tag.before.id,
                twin_core_design_tag(TwinCoreDesignTag::Heated)
            );
            assert_eq!(tag.kind, CombatTagChangeKind::Added);
        }
        _ => panic!("Expected Tag transition"),
    }
}
