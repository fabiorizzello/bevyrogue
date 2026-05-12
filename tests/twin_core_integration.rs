use bevy::prelude::*;

use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kernel::{
    CombatBeatId, CombatKernelRegistry, CombatKernelTransition, CombatTagChangeKind,
    CombatTagState, CombatTagTransition,
};
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::twin_core::{
    TwinCoreDesignTag, TwinCoreHook, TwinCoreState, apply_twin_core_transitions_system,
    twin_core_design_tag,
};
use bevyrogue::combat::types::{SkillId, UnitId};
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::skills_ron::{SkillBook, SkillDef};
use bevyrogue::data::units_ron::{UnitDef, UnitRoster};

fn load_roster() -> UnitRoster {
    ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron")
}

fn load_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
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

fn build_app() -> App {
    let mut app = App::new();
    let mut registry = CombatKernelRegistry::new();
    registry.register(TwinCoreHook);

    app.insert_resource(registry)
        .init_resource::<TwinCoreState>()
        .insert_resource(CombatState::default())
        .insert_resource(SpPool {
            current: 10,
            max: 10,
        })
        .insert_resource(ActionLog::default())
        .add_message::<CombatEvent>()
        .add_systems(Update, apply_twin_core_transitions_system);

    app
}

fn added_twin_core_tag(tag: TwinCoreDesignTag) -> CombatKernelTransition {
    let tag = twin_core_design_tag(tag);
    CombatKernelTransition::Tag(CombatTagTransition {
        before: CombatTagState::new(tag.clone(), 3),
        after: CombatTagState::new(tag, 3),
        kind: CombatTagChangeKind::Added,
    })
}

fn pump_kernel_transition(
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
        });
    }

    app.update();
}

fn capture_snapshot_string(app: &mut App) -> String {
    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should capture");
    format_validation_snapshot(&snapshot)
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

    assert_eq!(skill_def(&skills, "baby_flame").id, agumon.basic_skill);
    assert_eq!(skill_def(&skills, "agumon_ult").id, agumon.ultimate_skill);
    assert_eq!(skill_def(&skills, "bubble_blast").id, gabumon.basic_skill);
    assert_eq!(skill_def(&skills, "gabumon_ult").id, gabumon.ultimate_skill);
    assert_eq!(skill_def(&skills, "greymon_basic").id, greymon.basic_skill);
    assert_eq!(
        skill_def(&skills, "garurumon_basic").id,
        garurumon.basic_skill
    );

    let mut app = build_app();
    for def in [agumon, gabumon, greymon, garurumon, ogremon] {
        spawn_snapshot_unit(&mut app, def);
    }

    let source = agumon.id;
    let target = ogremon.id;

    pump_kernel_transition(
        &mut app,
        added_twin_core_tag(TwinCoreDesignTag::Heated),
        source,
        target,
    );
    let after_fire = capture_snapshot_string(&mut app);
    assert!(after_fire.contains("twin_core=cr=1"), "{after_fire}");
    assert!(after_fire.contains("spark_targets=[]"), "{after_fire}");
    assert!(after_fire.contains("last=build(1)"), "{after_fire}");

    pump_kernel_transition(
        &mut app,
        added_twin_core_tag(TwinCoreDesignTag::Chilled),
        source,
        target,
    );
    let after_ice = capture_snapshot_string(&mut app);
    assert!(after_ice.contains("twin_core=cr=2"), "{after_ice}");
    assert!(after_ice.contains("spark_targets=[]"), "{after_ice}");
    assert!(after_ice.contains("last=build(1)"), "{after_ice}");

    pump_kernel_transition(
        &mut app,
        added_twin_core_tag(TwinCoreDesignTag::ThermalSpark),
        source,
        target,
    );
    let after_spark = capture_snapshot_string(&mut app);
    assert!(
        after_spark.contains(&format!("spark_targets=[{}]", target.0)),
        "{after_spark}"
    );
    assert!(after_spark.contains("last=spark(1)"), "{after_spark}");

    pump_kernel_transition(
        &mut app,
        CombatKernelTransition::Beat(CombatBeatId::Damage),
        source,
        target,
    );
    let after_beat = capture_snapshot_string(&mut app);
    assert!(after_beat.contains("twin_core=cr=0"), "{after_beat}");
    assert!(after_beat.contains("spark_targets=[]"), "{after_beat}");
    assert!(after_beat.contains("burst_guard=true"), "{after_beat}");
    assert!(after_beat.contains("last=twin-burst(1)"), "{after_beat}");
    assert!(after_beat.contains("units=["), "{after_beat}");

    let final_snapshot = capture_snapshot_string(&mut app);
    assert!(
        final_snapshot.contains("twin_core=cr=0"),
        "{final_snapshot}"
    );
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

    let mut app = build_app();
    spawn_snapshot_unit(&mut app, agumon_def);
    spawn_snapshot_unit(&mut app, ogremon_def);

    // 1. Resolve Baby Flame (Agumon Basic)
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
        .expect("should resolve baby_flame");

    assert_eq!(resolved.skill_id, SkillId("baby_flame".into()));
    assert!(!resolved.custom_signals.is_empty());
    assert_eq!(resolved.custom_signals[0].owner(), "agumon");
    assert_eq!(resolved.custom_signals[0].signal(), "apply_heated");

    // 2. Dispatch transitions
    let transitions = bevyrogue::combat::blueprints::transitions_for_action(&resolved);
    assert_eq!(transitions.len(), 1);

    match &transitions[0] {
        bevyrogue::combat::kernel::CombatKernelTransition::Tag(tag) => {
            assert_eq!(tag.before.id, bevyrogue::combat::twin_core::twin_core_design_tag(bevyrogue::combat::twin_core::TwinCoreDesignTag::Heated));
            assert_eq!(tag.kind, bevyrogue::combat::kernel::CombatTagChangeKind::Added);
        }
        _ => panic!("Expected Tag transition"),
    }
}
