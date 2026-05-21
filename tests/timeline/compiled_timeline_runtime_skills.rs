//! Runtime-execution parity for canonical compiled timelines.
//!
//! Merged from three former single-skill files
//! (`compiled_timeline_petit_thunder.rs`, `compiled_timeline_tohakken.rs`,
//! and the `baby_flame_timeline_path_…` test in
//! `compiled_timeline_active_canon.rs`). Each case fires one canonical skill
//! through the compiled-timeline path and asserts the event order +
//! post-state invariants per the skill's contract.
//!
//! Differences across cases reduce to:
//! `(caster_attr, skill_id, owner, signal, target_toughness, event_order,
//! post_asserts)`.

use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;
use bevyrogue::combat::{
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    blueprints::register_all_blueprint_exts,
    events::{CombatEvent, CombatEventKind},
    kernel::CombatKernelTransition,
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    runtime::timeline::TimelineLibrary,
    runtime::{ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins},
    sp::SpPool,
    state::CombatState,
    status_effect::{StatusBag, StatusEffectKind},
    stun::Stunned,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, apply_av_ops_system, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skill_timeline::compile_skill_book_timelines,
    skills_ron::{SkillBook, validate_skill_book},
};
use rstest::rstest;

const CASTER: UnitId = UnitId(1);
const TARGET: UnitId = UnitId(2);

/// Predicate over a `CombatEventKind`. Returns true if the event matches.
type EventPred = fn(&CombatEventKind) -> bool;

/// Per-case post-state assertion. Receives the live `App` and the spawned
/// target `Entity`; panics on failure.
type PostAssert = fn(&mut App, Entity);

fn build_app(book: &SkillBook, seed: u64) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 99,
            max: 99,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(seed))
        .insert_resource(TimelineLibrary::<String>::default())
        .init_resource::<SignalBus>()
        .init_resource::<ExtRegistries>()
        .init_resource::<SignalTaxonomy>()
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, (resolve_action_system, apply_av_ops_system).chain());

    {
        let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
        register_kernel_builtins(&mut regs);
        register_all_blueprint_exts(&mut regs);
        let compiled = compile_skill_book_timelines(book, &regs)
            .expect("canonical timeline book must compile");
        app.world_mut()
            .resource_mut::<TimelineLibrary<String>>()
            .timelines = compiled;
    }

    app
}

fn spawn_caster(app: &mut App, name: &str, skill_id: &str, attr: Attribute) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: CASTER,
                name: name.into(),
                hp_max: 500,
                hp_current: 500,
                attribute: attr,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            UnitSkills {
                basic: SkillId(skill_id.into()),
                skills: vec![SkillId(skill_id.into())],
                ultimate: SkillId(skill_id.into()),
                follow_up: None,
            },
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 10,
            },
            Toughness::new(50, vec![]),
            StatusBag::default(),
        ))
        .id()
}

fn spawn_target(app: &mut App, toughness: Toughness) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id: TARGET,
                name: "Target".into(),
                hp_max: 200,
                hp_current: 200,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            ActionValue(MAX_AV),
            toughness,
            StatusBag::default(),
        ))
        .id()
}

fn fire_skill(app: &mut App, skill_id: &str) {
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: CASTER,
        skill_id: SkillId(skill_id.into()),
        target: TARGET,
    });
    app.update();
}

fn collect_events(app: &App, cursor: &mut MessageCursor<CombatEvent>) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn find_pos(events: &[CombatEvent], pred: EventPred, label: &str) -> usize {
    events
        .iter()
        .position(|e| pred(&e.kind))
        .unwrap_or_else(|| {
            panic!(
                "expected `{label}` event not found; events={:?}",
                events
                    .iter()
                    .map(|e| format!("{:?}", e.kind))
                    .collect::<Vec<_>>()
            )
        })
}

// ── case predicates ────────────────────────────────────────────────────────

fn is_damage(k: &CombatEventKind) -> bool {
    matches!(k, CombatEventKind::OnDamageDealt { .. })
}
fn is_break(k: &CombatEventKind) -> bool {
    matches!(k, CombatEventKind::OnBreak { .. })
}
fn is_applied(k: &CombatEventKind) -> bool {
    matches!(k, CombatEventKind::OnActionApplied)
}
fn is_resolved(k: &CombatEventKind) -> bool {
    matches!(k, CombatEventKind::OnActionResolved)
}
fn is_paralyzed(k: &CombatEventKind) -> bool {
    matches!(k, CombatEventKind::OnStatusApplied { kind } if *kind == StatusEffectKind::Paralyzed)
}
fn is_blessed(k: &CombatEventKind) -> bool {
    matches!(k, CombatEventKind::OnStatusApplied { kind } if *kind == StatusEffectKind::Blessed)
}
fn is_delay50(k: &CombatEventKind) -> bool {
    matches!(k, CombatEventKind::DelayTurn { amount_pct: 50, .. })
}
fn is_signal_tentomon_static(k: &CombatEventKind) -> bool {
    matches!(
        k,
        CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint { owner, name, .. }
        } if owner == "tentomon" && name == "build_static_charge"
    )
}
fn is_signal_agumon_heated(k: &CombatEventKind) -> bool {
    matches!(
        k,
        CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint { owner, name, .. }
        } if owner == "agumon" && name == "apply_heated"
    )
}

// ── post-state asserts ──────────────────────────────────────────────────────

fn assert_petit_thunder_post(app: &mut App, target: Entity) {
    let target_unit = app
        .world()
        .get::<Unit>(target)
        .expect("target unit missing");
    assert!(
        target_unit.hp_current < 200,
        "timeline skill should deal damage"
    );
    assert!(
        app.world().get::<Stunned>(target).is_some(),
        "BreakToughness should stun the target"
    );
    let status_bag = app
        .world()
        .get::<StatusBag>(target)
        .expect("target status bag missing");
    assert!(
        status_bag.has(&StatusEffectKind::Paralyzed),
        "Paralyzed must be applied"
    );
}

fn assert_tohakken_post(app: &mut App, target: Entity) {
    let target_unit = app
        .world()
        .get::<Unit>(target)
        .expect("target unit missing");
    assert!(
        target_unit.hp_current < 200,
        "timeline skill should deal damage"
    );
    assert_eq!(
        app.world()
            .get::<ActionValue>(target)
            .expect("target AV missing")
            .0,
        5000,
        "DelayTurn should flow through apply_av_ops_system"
    );
    let mut q = app
        .world_mut()
        .query_filtered::<&StatusBag, With<UnitSkills>>();
    let caster_bag = q
        .iter(app.world())
        .next()
        .expect("caster status bag missing");
    assert!(
        caster_bag.has(&StatusEffectKind::Blessed),
        "caster must receive Blessed"
    );
}

fn assert_baby_flame_post(app: &mut App, target: Entity) {
    let target_unit = app
        .world()
        .get::<Unit>(target)
        .expect("target unit missing");
    assert!(
        target_unit.hp_current < 200,
        "baby_flame must deal damage via timeline path"
    );
}

// ── parametric matrix ──────────────────────────────────────────────────────

/// Case 1 — Tentomon `petit_thunder`: damage → break → paralyze status → signal → applied → resolved.
///         Post: hp<200, Stunned, Paralyzed.
#[rstest]
#[case::petit_thunder(
    "Tentomon",
    Attribute::Vaccine,
    "petit_thunder",
    "tentomon",
    "build_static_charge",
    Toughness::new(8, vec![DamageTag::Electric]),
    &[("damage", is_damage as EventPred),
      ("break", is_break),
      ("paralyzed", is_paralyzed),
      ("signal", is_signal_tentomon_static),
      ("applied", is_applied),
      ("resolved", is_resolved)],
    assert_petit_thunder_post as PostAssert,
)]
/// Case 2 — Renamon `renamon_ult` (tohakken): damage → delay50 → blessed → applied → resolved.
///         Post: hp<200, target AV==5000, caster has Blessed.
#[case::renamon_ult_tohakken(
    "Renamon",
    Attribute::Vaccine,
    "renamon_ult",
    "renamon",
    "commit_precision_press",
    Toughness::new(30, vec![]),
    &[("damage", is_damage as EventPred),
      ("delay50", is_delay50),
      ("blessed", is_blessed),
      ("applied", is_applied),
      ("resolved", is_resolved)],
    assert_tohakken_post as PostAssert,
)]
/// Case 3 — Agumon `baby_flame`: damage → break → signal.
///         Post: hp<200.
#[case::baby_flame(
    "Caster",
    Attribute::Vaccine,
    "baby_flame",
    "agumon",
    "apply_heated",
    Toughness::new(10, vec![DamageTag::Fire]),
    &[("damage", is_damage as EventPred),
      ("break", is_break),
      ("signal", is_signal_agumon_heated)],
    assert_baby_flame_post as PostAssert,
)]
fn compiled_timeline_runtime_path_emits_canonical_event_order_and_post_state(
    #[case] caster_name: &str,
    #[case] caster_attr: Attribute,
    #[case] skill_id: &str,
    #[case] signal_owner: &'static str,
    #[case] signal_name: &'static str,
    #[case] target_toughness: Toughness,
    #[case] expected_order: &[(&str, EventPred)],
    #[case] post_assert: PostAssert,
) {
    let book = bevyrogue::data::aggregate_skill_book();
    validate_skill_book(&book).expect("canonical skills.ron must validate");

    let mut app = build_app(&book, 7);
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register(signal_owner, signal_name);
    let _caster = spawn_caster(&mut app, caster_name, skill_id, caster_attr);
    let target = spawn_target(&mut app, target_toughness);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor_current();
    fire_skill(&mut app, skill_id);

    let events = collect_events(&app, &mut cursor);

    let positions: Vec<usize> = expected_order
        .iter()
        .map(|(label, pred)| find_pos(&events, *pred, label))
        .collect();

    for window in positions.windows(2) {
        let (a, b) = (window[0], window[1]);
        assert!(
            a < b,
            "event order violated for `{skill_id}`: positions {positions:?}, events={:?}",
            events
                .iter()
                .map(|e| format!("{:?}", e.kind))
                .collect::<Vec<_>>()
        );
    }

    post_assert(&mut app, target);
}
