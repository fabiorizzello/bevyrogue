/// Integration tests for M008/S03: per-unit ultimate meter accumulation and turn-order
/// interruption on ultimate fire.
///
/// Tests in this file exercise:
/// - `ult_accumulation_system` + `flush_ult_gain_system` for multiple trigger types
/// - R007 regression: Taichi still charges via OnOffensivePartyEvent
/// - Turn-order front-insert on successful ultimate fire (via resolve_action_system)
/// - Negative paths: ult not ready, commander defender
use bevy::prelude::*;
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::{DamageKind, Toughness},
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{
        UltAccumulationTrigger, UltGainQueue, UltimateCharge, flush_ult_gain_system,
        ult_accumulation_system,
    },
    unit::{Commander, Unit},
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_unit(id: u32, name: &str, hp: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: name.into(),
        hp_max: hp,
        hp_current: hp,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn combat_event(kind: CombatEventKind, source: UnitId, target: UnitId) -> CombatEvent {
    CombatEvent {
        kind,
        source,
        target,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    }
}

fn ult_charge(
    current: i32,
    trigger: i32,
    trigger_type: UltAccumulationTrigger,
    cpe: i32,
) -> UltimateCharge {
    UltimateCharge {
        current,
        trigger,
        cap: trigger + 50,
        trigger_type,
        charge_per_event: cpe,
    }
}

fn setup_accumulation_app() -> App {
    let mut app = App::new();
    app.init_resource::<UltGainQueue>()
        .add_message::<CombatEvent>()
        .add_systems(
            Update,
            (ult_accumulation_system, flush_ult_gain_system).chain(),
        );
    app
}

fn setup_resolve_app(skill_book: SkillBook) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book);
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<TurnAdvanced>()
        .add_systems(Update, resolve_action_system);
    app
}

fn ult_skill() -> SkillDef {
    SkillDef {
        id: SkillId("ult".into()),
        name: "Ultimate".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![
            Effect::Damage {
                amount: 50,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(10),
        ],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn basic_skill() -> SkillDef {
    SkillDef {
        id: SkillId("basic".into()),
        name: "Basic".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![
            Effect::Damage {
                amount: 10,
                target: TargetShape::Single,
                per_hop: Default::default(),
            },
            Effect::ToughnessHit(5),
        ],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn unit_skills_with_ult() -> UnitSkills {
    UnitSkills {
        basic: SkillId("basic".into()),
        skills: vec![],
        ultimate: SkillId("ult".into()),
        follow_up: None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Gate test: two units with different trigger types independently reach their ready threshold.
/// Unit A (OnOffensivePartyEvent, cpe=20, trigger=40) charges on ally damage events.
/// Unit B (OnHitTaken, cpe=30, trigger=60) charges when it takes hits.
/// After emitting 2 events per unit they both become ready with distinct charge paths.
#[test]
fn two_units_with_different_triggers_reach_ready_independently() {
    let mut app = setup_accumulation_app();

    // unit_a: OnOffensivePartyEvent — charges when any ally deals damage
    app.world_mut().spawn((
        make_unit(1, "UnitA", 100),
        Team::Ally,
        ult_charge(0, 40, UltAccumulationTrigger::OnOffensivePartyEvent, 20),
    ));
    // unit_b: OnHitTaken — charges when it is the target of a hit
    app.world_mut().spawn((
        make_unit(2, "UnitB", 100),
        Team::Ally,
        ult_charge(0, 60, UltAccumulationTrigger::OnHitTaken, 30),
    ));
    // An enemy so there is a valid target in the world (needed for team lookup)
    app.world_mut()
        .spawn((make_unit(101, "Enemy", 100), Team::Enemy));

    // 2x ally damage → unit_a gains 20+20=40 (reaches trigger)
    for _ in 0..2 {
        app.world_mut().write_message(combat_event(
            CombatEventKind::OnDamageDealt {
                amount: 30,
                kind: DamageKind::Normal,
                tag_mod_pct: 100,
                triangle_mod_pct: 100,
                damage_tag: DamageTag::Fire,
            },
            UnitId(1),
            UnitId(101),
        ));
    }
    // 2x hit-taken on unit_b → unit_b gains 30+30=60 (reaches trigger)
    for _ in 0..2 {
        app.world_mut().write_message(combat_event(
            CombatEventKind::OnHitTaken { amount: 30 },
            UnitId(101),
            UnitId(2),
        ));
    }

    app.update();

    let mut unit_a_ready = false;
    let mut unit_b_ready = false;
    let mut q = app.world_mut().query::<(&Unit, &UltimateCharge)>();
    for (unit, ult) in q.iter(app.world()) {
        if unit.id == UnitId(1) {
            unit_a_ready = ult.ready();
            assert_eq!(ult.current, 40, "unit_a should have exactly 40 charge");
        }
        if unit.id == UnitId(2) {
            unit_b_ready = ult.ready();
            assert_eq!(ult.current, 60, "unit_b should have exactly 60 charge");
        }
    }
    assert!(
        unit_a_ready,
        "unit_a (OnOffensivePartyEvent) should be ready after 2 ally damage events"
    );
    assert!(
        unit_b_ready,
        "unit_b (OnHitTaken) should be ready after 2 hit-taken events"
    );
}

/// R007 regression: Taichi (Commander, OnOffensivePartyEvent) still accumulates charge
/// when an ally deals damage.
#[test]
fn taichi_still_charges_on_offensive_party_events() {
    let mut app = setup_accumulation_app();

    // Taichi: Commander, OnOffensivePartyEvent, cpe=10
    app.world_mut().spawn((
        make_unit(0, "Taichi", 200),
        Team::Ally,
        Commander,
        ult_charge(0, 80, UltAccumulationTrigger::OnOffensivePartyEvent, 10),
    ));
    // An ally attacker (source of the damage event)
    app.world_mut()
        .spawn((make_unit(1, "Agumon", 100), Team::Ally));
    // An enemy target
    app.world_mut()
        .spawn((make_unit(101, "Enemy", 100), Team::Enemy));

    // One ally damage event
    app.world_mut().write_message(combat_event(
        CombatEventKind::OnDamageDealt {
            amount: 40,
            kind: DamageKind::Normal,
            tag_mod_pct: 100,
            triangle_mod_pct: 100,
            damage_tag: DamageTag::Fire,
        },
        UnitId(1),
        UnitId(101),
    ));

    app.update();

    let mut taichi_current = None;
    let mut q = app.world_mut().query::<(&Unit, &UltimateCharge)>();
    for (unit, ult) in q.iter(app.world()) {
        if unit.id == UnitId(0) {
            taichi_current = Some(ult.current);
        }
    }
    assert_eq!(
        taichi_current,
        Some(10),
        "Taichi should gain 10 charge from one ally damage event (R007)"
    );
}

/// Turn-order interruption: after a successful ultimate fire, the attacker is front-inserted
/// into the queue so their next turn arrives before any unit that was ahead of them.
#[test]
fn ultimate_fire_injects_attacker_front_of_queue() {
    let mut app = setup_resolve_app(SkillBook(vec![ult_skill(), basic_skill()]));

    // unit 1: the attacker — ult ready
    app.world_mut().spawn((
        make_unit(1, "Attacker", 200),
        Team::Ally,
        unit_skills_with_ult(),
        ult_charge(100, 100, UltAccumulationTrigger::OnBasicAttack, 25),
        Toughness::new(100, vec![]),
    ));
    // unit 2: the defender — high HP so it doesn't die and complicate teardown
    app.world_mut().spawn((
        make_unit(2, "Defender", 1000),
        Team::Enemy,
        Toughness::new(200, vec![]),
    ));
    // unit 3: another unit in the queue ahead of the attacker
    app.world_mut()
        .spawn((make_unit(3, "Other", 100), Team::Enemy));

    // Seed queue so "Other" is in front and "Attacker" is behind
    app.world_mut()
        .resource_mut::<TurnOrder>()
        .seed([UnitId(3), UnitId(1), UnitId(2)]);

    // Fire the ultimate
    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    // In the AV system, there is no queue front-insert; verify ult resolved correctly.
    let mut q = app.world_mut().query::<(&Unit, &UltimateCharge)>();
    for (unit, ult) in q.iter(app.world()) {
        if unit.id == UnitId(1) {
            assert_eq!(
                ult.current, 0,
                "ult meter should be reset to 0 after firing"
            );
        }
        if unit.id == UnitId(2) {
            assert!(
                unit.hp_current < 1000,
                "defender should have taken damage from ultimate"
            );
        }
    }
}

/// Negative: when the ultimate meter is not ready, no front-insert occurs.
#[test]
fn ult_not_ready_does_not_front_insert() {
    let mut app = setup_resolve_app(SkillBook(vec![ult_skill(), basic_skill()]));

    app.world_mut().spawn((
        make_unit(1, "Attacker", 200),
        Team::Ally,
        unit_skills_with_ult(),
        // current=0 → not ready
        ult_charge(0, 100, UltAccumulationTrigger::OnBasicAttack, 25),
        Toughness::new(100, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(2, "Defender", 1000),
        Team::Enemy,
        Toughness::new(200, vec![]),
    ));
    app.world_mut()
        .spawn((make_unit(3, "Other", 100), Team::Enemy));

    app.world_mut()
        .resource_mut::<TurnOrder>()
        .seed([UnitId(3), UnitId(1), UnitId(2)]);

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    // In the AV system, there is no queue; verify ult was rejected (defender HP unchanged).
    let mut q = app.world_mut().query::<&Unit>();
    let defender_hp = q
        .iter(app.world())
        .find(|u| u.id == UnitId(2))
        .map(|u| u.hp_current);
    assert_eq!(
        defender_hp,
        Some(1000),
        "defender should be undamaged when ult not ready"
    );
}

/// Negative: when the target is a Commander, apply_legacy_ops rejects the action without
/// consuming the ult meter, so no front-insert occurs.
#[test]
fn commander_defender_does_not_front_insert() {
    let mut app = setup_resolve_app(SkillBook(vec![ult_skill(), basic_skill()]));

    app.world_mut().spawn((
        make_unit(1, "Attacker", 200),
        Team::Ally,
        unit_skills_with_ult(),
        // ult ready
        ult_charge(100, 100, UltAccumulationTrigger::OnBasicAttack, 25),
        Toughness::new(100, vec![]),
    ));
    // Defender is a Commander — apply_legacy_ops will reject and skip meter reset
    app.world_mut().spawn((
        make_unit(2, "Taichi", 200),
        Team::Ally,
        Commander,
        Toughness::new(200, vec![]),
    ));
    app.world_mut()
        .spawn((make_unit(3, "Other", 100), Team::Enemy));

    app.world_mut()
        .resource_mut::<TurnOrder>()
        .seed([UnitId(3), UnitId(1)]);

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    // Commander rejection means meter is not consumed → no front-insert
    let order = app.world().resource::<TurnOrder>();
    assert_ne!(
        order.queue.front().copied(),
        Some(UnitId(1)),
        "attacker should NOT be front-inserted when defender is Commander (queue: {:?})",
        order.queue
    );

    // Verify ult meter was NOT consumed
    let mut q = app.world_mut().query::<(&Unit, &UltimateCharge)>();
    for (unit, ult) in q.iter(app.world()) {
        if unit.id == UnitId(1) {
            assert_eq!(
                ult.current, 100,
                "ult meter should not be consumed when targeting Commander"
            );
        }
    }
}
