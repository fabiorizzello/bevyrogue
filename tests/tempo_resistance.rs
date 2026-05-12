//! Integration tests for Tempo Resistance and Minimum Action Threshold.
//!
//! T02: pure-logic curve tests (no Bevy world needed).
//! T03: boss-spawn scenario tests (TempoResistance wired via UnitDef.tempo_resistant).

use bevy::prelude::*;
use bevyrogue::combat::av::{ActionValue, MAX_AV};
use bevyrogue::combat::resistance::{
    MIN_ACTION_THRESHOLD_AV, TempoResistance, apply_av_change, compute_av_change,
};

// ──────────────────────────────────────────────────────────────────────────────
// Pure-logic tests (no Bevy app)
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn tempo_resistance_multiplier_curve() {
    let mut r = TempoResistance::default();
    assert_eq!(r.multiplier(), 1.0, "hit 0 → 100%");
    r.record_delay_hit();
    assert_eq!(r.multiplier(), 0.5, "hit 1 → 50%");
    r.record_delay_hit();
    assert_eq!(r.multiplier(), 0.25, "hit 2 → 25%");
    r.record_delay_hit();
    assert_eq!(r.multiplier(), 0.25, "hit 3+ stays at 25%");
}

/// Parameterised: each successive Delay on the same unit gets attenuated.
#[test]
fn three_consecutive_delays_show_diminishing_returns() {
    let mut r = TempoResistance::default();
    // -20% of MAX_AV (10000) = -2000 raw
    let d1 = compute_av_change(-20, Some(&r)); // 100% → -2000
    r.record_delay_hit();
    let d2 = compute_av_change(-20, Some(&r)); // 50%  → -1000
    r.record_delay_hit();
    let d3 = compute_av_change(-20, Some(&r)); // 25%  → -500

    assert_eq!(d1, -2000, "first delay: 100%");
    assert_eq!(d2, -1000, "second delay: 50%");
    assert_eq!(d3, -500, "third delay: 25%");
}

#[test]
fn compute_av_change_advance_bypasses_resistance() {
    // Positive advance is unaffected even at max resistance stack
    let r = TempoResistance { hit_count: 99 };
    assert_eq!(
        compute_av_change(20, Some(&r)),
        2000,
        "advance ignores resistance"
    );
}

#[test]
fn compute_av_change_no_resistance_component() {
    // Without a TempoResistance component, delays are full-strength
    assert_eq!(compute_av_change(-30, None), -3000);
}

#[test]
fn apply_av_change_records_hit_and_updates_av() {
    let mut av = ActionValue(MAX_AV / 2); // 5000
    let mut r = TempoResistance::default();

    let delta = apply_av_change(&mut av, Some(&mut r), -20); // -2000 (100%)
    assert_eq!(delta, -2000);
    assert_eq!(av.0, 3000);
    assert_eq!(r.hit_count, 1, "resistance stack advanced after delay");

    let delta2 = apply_av_change(&mut av, Some(&mut r), -20); // -1000 (50%)
    assert_eq!(delta2, -1000);
    assert_eq!(av.0, 2000);
    assert_eq!(r.hit_count, 2);
}

#[test]
fn apply_av_change_clamps_to_min_action_threshold() {
    // Massive delay: -200% = -20000, but floor is -MIN_ACTION_THRESHOLD_AV = -15000
    let mut av = ActionValue(0);
    let mut r = TempoResistance::default();
    apply_av_change(&mut av, Some(&mut r), -200);
    assert_eq!(av.0, -MIN_ACTION_THRESHOLD_AV, "AV clamped to floor");
}

#[test]
fn apply_av_change_clamps_without_resistance_too() {
    let mut av = ActionValue(0);
    apply_av_change(&mut av, None, -200); // raw -20000 → clamped
    assert_eq!(av.0, -MIN_ACTION_THRESHOLD_AV);
}

#[test]
fn apply_av_change_advance_does_not_exceed_max_av() {
    let mut av = ActionValue(MAX_AV - 500);
    let delta = apply_av_change(&mut av, None, 20); // +2000, but cap at MAX_AV
    assert_eq!(av.0, MAX_AV);
    assert_eq!(delta, 500, "delta clamped to headroom");
}

// ──────────────────────────────────────────────────────────────────────────────
// Bevy integration: apply_turn_advance_system wires CombatEvent → ActionValue
// ──────────────────────────────────────────────────────────────────────────────

use bevy::ecs::message::Messages;
use bevyrogue::combat::av::ActionValueUpdated;
use bevyrogue::combat::bootstrap::spawn_unit_from_def;
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::turn_order::{TurnAdvanced, TurnOrder};
use bevyrogue::combat::turn_system::{ActionIntent, apply_turn_advance_system};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;

fn uid(n: u32) -> UnitId {
    UnitId(n)
}

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<TurnOrder>();
    app.init_resource::<CombatState>();
    app.add_message::<TurnAdvanced>();
    app.add_message::<ActionValueUpdated>();
    app.add_message::<CombatEvent>();
    app.add_message::<ActionIntent>();
    app.add_systems(Update, apply_turn_advance_system);
    app
}

fn spawn_unit_with_resistance(world: &mut World, id: UnitId) -> Entity {
    world
        .spawn((
            Unit {
                id,
                name: format!("Boss_{}", id.0),
                hp_max: 1000,
                hp_current: 1000,
                attribute: bevyrogue::combat::types::Attribute::Free,
                resists: vec![],
                evo_stage: bevyrogue::combat::types::EvoStage::Child,
            },
            ActionValue(MAX_AV / 2), // start at mid-turn (5000)
            Team::Enemy,
            TempoResistance::default(),
        ))
        .id()
}

fn send_combat_event(app: &mut App, event: CombatEvent) {
    app.world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .write(event);
}

#[test]
fn system_applies_delay_with_resistance_via_combat_event() {
    let mut app = setup_app();
    let entity = spawn_unit_with_resistance(app.world_mut(), uid(10));

    send_combat_event(
        &mut app,
        CombatEvent {
            kind: CombatEventKind::TurnAdvance {
                target: uid(10),
                amount_pct: -20,
            },
            source: uid(1),
            target: uid(10),
            follow_up_depth: 0,
        },
    );
    app.update();

    let av = app.world().entity(entity).get::<ActionValue>().unwrap();
    let res = app.world().entity(entity).get::<TempoResistance>().unwrap();
    assert_eq!(av.0, 3000, "AV reduced by 2000 (100% of 20%)");
    assert_eq!(res.hit_count, 1, "resistance stack incremented");

    // Second hit: 50% effective → -1000
    send_combat_event(
        &mut app,
        CombatEvent {
            kind: CombatEventKind::TurnAdvance {
                target: uid(10),
                amount_pct: -20,
            },
            source: uid(1),
            target: uid(10),
            follow_up_depth: 0,
        },
    );
    app.update();

    let av2 = app.world().entity(entity).get::<ActionValue>().unwrap();
    let res2 = app.world().entity(entity).get::<TempoResistance>().unwrap();
    assert_eq!(av2.0, 2000, "AV reduced by 1000 (50% of 20%)");
    assert_eq!(res2.hit_count, 2);
}

#[test]
fn system_applies_advance_without_touching_resistance_stack() {
    let mut app = setup_app();
    let entity = spawn_unit_with_resistance(app.world_mut(), uid(11));

    send_combat_event(
        &mut app,
        CombatEvent {
            kind: CombatEventKind::TurnAdvance {
                target: uid(11),
                amount_pct: 20,
            },
            source: uid(1),
            target: uid(11),
            follow_up_depth: 0,
        },
    );
    app.update();

    let av = app.world().entity(entity).get::<ActionValue>().unwrap();
    let res = app.world().entity(entity).get::<TempoResistance>().unwrap();
    // Unit started at MAX_AV/2 (5000), +20% = +2000 → 7000
    assert_eq!(av.0, MAX_AV / 2 + 2000, "advance raises AV by 2000");
    assert_eq!(
        res.hit_count, 0,
        "advance does not increment resistance stack"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T03: Boss-spawn scenario — TempoResistance wired via UnitDef.tempo_resistant
// ──────────────────────────────────────────────────────────────────────────────

fn devimon_def() -> bevyrogue::data::units_ron::UnitDef {
    use bevyrogue::combat::team::Team;
    use bevyrogue::combat::types::*;
    use bevyrogue::combat::ultimate::UltAccumulationTrigger;

    bevyrogue::data::units_ron::UnitDef {
        id: UnitId(101),
        name: "Devimon".into(),
        role_tags: vec!["boss".into(), "dark".into()],
        signature_traits: vec!["evil".into(), "dark".into()],
        hp_max: 500,
        attribute: Attribute::Virus,
        team: Team::Enemy,
        basic_damage_tag: DamageTag::Dark,
        basic_skill: SkillId("enemy_skill_fire".into()),
        skill_ids: vec![SkillId("enemy_skill_fire".into())],
        ultimate_skill: SkillId("enemy_ult_fire".into()),
        follow_up: None,
        enemy_traits: vec![],
        charged_attack: None,
        form_identity: None,
        twin_core: Default::default(),
        holy_support: Default::default(),
        resists: vec![DamageTag::Fire, DamageTag::Ice],
        toughness_max: 100,
        weaknesses: vec![DamageTag::Light],
        ultimate_trigger: 100,
        ultimate_cap: 150,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
        ultimate_charge_per_event: 25,
        speed: 80,
        evo_stage: EvoStage::Child,
        evo_line: EvoLineId("devimon_line".into()),
        evolves_to: vec![],
        tempo_resistant: true,
        toughness_category: Default::default(),
    }
}

fn ally_def() -> bevyrogue::data::units_ron::UnitDef {
    use bevyrogue::combat::team::Team;
    use bevyrogue::combat::types::*;
    use bevyrogue::combat::ultimate::UltAccumulationTrigger;

    bevyrogue::data::units_ron::UnitDef {
        id: UnitId(1),
        name: "Agumon".into(),
        role_tags: vec!["vanguard".into()],
        signature_traits: vec!["courage".into()],
        hp_max: 100,
        attribute: Attribute::Vaccine,
        team: Team::Ally,
        basic_damage_tag: DamageTag::Fire,
        basic_skill: SkillId("baby_flame".into()),
        skill_ids: vec![SkillId("baby_flame".into())],
        ultimate_skill: SkillId("agumon_ult".into()),
        follow_up: None,
        enemy_traits: vec![],
        charged_attack: None,
        form_identity: None,
        twin_core: Default::default(),
        holy_support: Default::default(),
        resists: vec![],
        toughness_max: 50,
        weaknesses: vec![DamageTag::Ice],
        ultimate_trigger: 100,
        ultimate_cap: 150,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
        ultimate_charge_per_event: 25,
        speed: 100,
        evo_stage: EvoStage::Child,
        evo_line: EvoLineId("agumon_line".into()),
        evolves_to: vec![UnitId(12)],
        tempo_resistant: false,
        toughness_category: Default::default(),
    }
}

/// A boss unit with `tempo_resistant: true` gets a `TempoResistance` component on spawn.
#[test]
fn boss_spawn_gets_tempo_resistance_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let def = devimon_def();
    let entity = spawn_unit_from_def(&mut app.world_mut().commands(), &def);
    app.update();

    assert!(
        app.world()
            .entity(entity)
            .get::<TempoResistance>()
            .is_some(),
        "boss with tempo_resistant: true must have TempoResistance component"
    );
    let res = app.world().entity(entity).get::<TempoResistance>().unwrap();
    assert_eq!(res.hit_count, 0, "fresh boss starts at 0 delay hits");
}

/// A regular ally unit with `tempo_resistant: false` does NOT get `TempoResistance`.
#[test]
fn ally_spawn_has_no_tempo_resistance_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let def = ally_def();
    let entity = spawn_unit_from_def(&mut app.world_mut().commands(), &def);
    app.update();

    assert!(
        app.world()
            .entity(entity)
            .get::<TempoResistance>()
            .is_none(),
        "ally with tempo_resistant: false must NOT have TempoResistance component"
    );
}

/// Three consecutive Delay events on a boss show 100→50→25% attenuation end-to-end.
///
/// This is the full pipeline: spawn_unit_from_def → CombatEvent bus → apply_turn_advance_system.
#[test]
fn boss_scenario_three_slow_hits_show_resistance_curve() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<TurnOrder>();
    app.init_resource::<CombatState>();
    app.add_message::<TurnAdvanced>();
    app.add_message::<ActionValueUpdated>();
    app.add_message::<CombatEvent>();
    app.add_message::<ActionIntent>();
    app.add_systems(Update, apply_turn_advance_system);

    // Spawn boss from def — TempoResistance should be inserted automatically.
    let def = devimon_def();
    let entity = spawn_unit_from_def(&mut app.world_mut().commands(), &def);
    // Flush commands so the entity is live, then override AV to a known value.
    app.update();
    // Give the boss a known starting AV so we can assert exact values.
    app.world_mut()
        .entity_mut(entity)
        .insert(ActionValue(MAX_AV / 2)); // 5000

    let boss_id = uid(101);

    // Hit 1: 100% of -20% = -2000 → AV 5000 → 3000
    send_combat_event(
        &mut app,
        CombatEvent {
            kind: CombatEventKind::TurnAdvance {
                target: boss_id,
                amount_pct: -20,
            },
            source: uid(1),
            target: boss_id,
            follow_up_depth: 0,
        },
    );
    app.update();
    assert_eq!(
        app.world().entity(entity).get::<ActionValue>().unwrap().0,
        3000,
        "hit 1: 100%"
    );
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<TempoResistance>()
            .unwrap()
            .hit_count,
        1
    );

    // Hit 2: 50% of -20% = -1000 → AV 3000 → 2000
    send_combat_event(
        &mut app,
        CombatEvent {
            kind: CombatEventKind::TurnAdvance {
                target: boss_id,
                amount_pct: -20,
            },
            source: uid(1),
            target: boss_id,
            follow_up_depth: 0,
        },
    );
    app.update();
    assert_eq!(
        app.world().entity(entity).get::<ActionValue>().unwrap().0,
        2000,
        "hit 2: 50%"
    );
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<TempoResistance>()
            .unwrap()
            .hit_count,
        2
    );

    // Hit 3: 25% of -20% = -500 → AV 2000 → 1500
    send_combat_event(
        &mut app,
        CombatEvent {
            kind: CombatEventKind::TurnAdvance {
                target: boss_id,
                amount_pct: -20,
            },
            source: uid(1),
            target: boss_id,
            follow_up_depth: 0,
        },
    );
    app.update();
    assert_eq!(
        app.world().entity(entity).get::<ActionValue>().unwrap().0,
        1500,
        "hit 3: 25%"
    );
    assert_eq!(
        app.world()
            .entity(entity)
            .get::<TempoResistance>()
            .unwrap()
            .hit_count,
        3
    );
}

/// Parse the canonical units.ron and verify Devimon is present with tempo_resistant: true.
#[test]
fn canonical_units_ron_contains_tempo_resistant_boss() {
    let roster: bevyrogue::data::units_ron::UnitRoster =
        ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron");

    let devimon = roster
        .0
        .iter()
        .find(|u| u.name == "Devimon")
        .expect("Devimon must be in units.ron");

    assert!(
        devimon.tempo_resistant,
        "Devimon.tempo_resistant must be true"
    );
    assert_eq!(devimon.team, Team::Enemy, "Devimon must be on Enemy team");
}
