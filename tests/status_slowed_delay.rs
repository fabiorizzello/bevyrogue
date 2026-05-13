/// Integration test for §H.2 Slowed semantics (S04/T02): first-apply pushes gauge −30%,
/// re-apply does NOT re-push (refresh_max_dur path only).
///
/// Scenario:
///   Attacker (Vaccine ally) casts a pure-ApplyStatus{Slowed, 3} skill on
///   Defender (Vaccine enemy, ActionValue=5000, no TempoResistance).
///
/// Assertions:
///   1. Exactly one TurnAdvance{target=defender_id, amount_pct=−30} event after first apply.
///   2. Defender AV reduced to 2000 (5000 − 3000, where 3000 = 30% of MAX_AV=10000, full hit).
///   3. Zero TurnAdvance events after second apply (refresh_max_dur only).
use bevy::prelude::*;
use bevy::ecs::message::Messages;
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    av::{ActionValue, ActionValueUpdated, MAX_AV},
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, apply_turn_advance_system, resolve_action_system},
    types::{Attribute, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
        TargetLife, TargetShape, TargetSide,
    },
};

fn slowed_skill() -> SkillDef {
    SkillDef {
        id: SkillId("apply_slowed".into()),
        name: "apply_slowed".into(),
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![Effect::ApplyStatus {
            kind: StatusEffectKind::Slowed,
            duration: 3,
        }],
        ..Default::default()
    }
}

fn setup_app() -> (App, Entity, Entity) {
    let mut app = App::new();

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![slowed_skill()]));

    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool { current: 100, max: 100 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(0))
        .add_message::<ActionIntent>()
        .add_message::<TurnAdvanced>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(
            Update,
            (resolve_action_system, apply_turn_advance_system).chain(),
        );

    let attacker = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(1),
                name: "Attacker".into(),
                hp_max: 10_000,
                hp_current: 10_000,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            UnitSkills {
                basic: SkillId("apply_slowed".into()),
                skills: vec![SkillId("apply_slowed".into())],
                ultimate: SkillId("apply_slowed".into()),
                follow_up: None,
            },
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 10,
            },
            Toughness::new(1_000, vec![]),
            StatusBag::default(),
        ))
        .id();

    let defender = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Defender".into(),
                hp_max: 10_000,
                hp_current: 10_000,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            // ActionValue set to mid-turn so delay is visible.
            ActionValue(MAX_AV / 2), // 5000
            Toughness::new(1_000, vec![]),
            StatusBag::default(),
        ))
        .id();

    (app, attacker, defender)
}

fn count_turn_advance_for(events: &[CombatEvent], target: UnitId, amount_pct: i32) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                &e.kind,
                CombatEventKind::TurnAdvance { target: t, amount_pct: a }
                    if *t == target && *a == amount_pct
            )
        })
        .count()
}

/// First apply of Slowed emits exactly one TurnAdvance{−30} and pushes AV by −3000.
/// Second apply (re-apply) emits zero additional TurnAdvance events.
#[test]
fn slowed_first_apply_delays_gauge_reapply_does_not() {
    let (mut app, _attacker, defender) = setup_app();
    let defender_id = app.world().get::<Unit>(defender).unwrap().id;

    // Cursor at write-head before any updates: reads only new messages per update.
    let mut cursor = app
        .world()
        .resource::<Messages<CombatEvent>>()
        .get_cursor_current();

    // ── First apply ──────────────────────────────────────────────────────────────
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("apply_slowed".into()),
        target: UnitId(2),
    });
    app.update();

    let events1: Vec<CombatEvent> = {
        let msgs = app.world().resource::<Messages<CombatEvent>>();
        cursor.read(msgs).cloned().collect()
    };

    let ta_count = count_turn_advance_for(&events1, defender_id, -30);
    assert_eq!(
        ta_count, 1,
        "first apply must emit exactly 1 TurnAdvance{{amount_pct:-30}}; got {ta_count}\nevents: {events1:?}"
    );

    // apply_turn_advance_system must have consumed the TurnAdvance event and updated AV.
    let av_after_first = app.world().get::<ActionValue>(defender).unwrap().0;
    assert_eq!(
        av_after_first, 2000,
        "defender AV must drop from 5000 to 2000 (5000 − 3000, full-strength −30%); got {av_after_first}"
    );

    // Confirm OnStatusApplied{Slowed} precedes TurnAdvance in the event stream.
    let status_applied_pos = events1.iter().position(|e| {
        matches!(&e.kind, CombatEventKind::OnStatusApplied { kind } if *kind == StatusEffectKind::Slowed)
    });
    let turn_advance_pos = events1.iter().position(|e| {
        matches!(&e.kind, CombatEventKind::TurnAdvance { amount_pct: -30, .. })
    });
    assert!(
        status_applied_pos < turn_advance_pos,
        "OnStatusApplied must precede TurnAdvance in event log"
    );

    // ── Second apply (re-apply) ───────────────────────────────────────────────────
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: UnitId(1),
        skill_id: SkillId("apply_slowed".into()),
        target: UnitId(2),
    });
    app.update();

    let events2: Vec<CombatEvent> = {
        let msgs = app.world().resource::<Messages<CombatEvent>>();
        cursor.read(msgs).cloned().collect()
    };

    let ta_count2 = count_turn_advance_for(&events2, defender_id, -30);
    assert_eq!(
        ta_count2, 0,
        "re-apply must emit zero TurnAdvance events (refresh_max_dur only); got {ta_count2}\nevents: {events2:?}"
    );
}
