use crate::common::app::turn_av_base_app;
/// Headless integration tests for M008/S01 enemy AI decision routing.
///
/// These tests exercise the full `advance_turn_system` path (TurnAdvanced →
/// pick_enemy_action → ActionIntent) without running resolve_action_system, so
/// no SkillBook asset is required — all inputs are inline Rust.
///
/// Enemy UnitIds use the 101+ range to avoid collisions with ally IDs 0–10 (MEM030).
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    kit::UnitSkills,
    team::Team,
    toughness::Toughness,
    turn_order::TurnAdvanced,
    turn_system::{
        ActionIntent, EnemyTurnRequestQueue, advance_turn_system, resolve_enemy_turn_action_system,
    },
    types::{Attribute, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup_app() -> App {
    let mut app = turn_av_base_app();
    app.init_resource::<EnemyTurnRequestQueue>().add_systems(
        Update,
        (advance_turn_system, resolve_enemy_turn_action_system).chain(),
    );
    app
}

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

/// Enemy skill set with a Basic, one Skill, and an Ultimate.
fn enemy_skills() -> UnitSkills {
    UnitSkills {
        basic: SkillId("enemy_basic".into()),
        skills: vec![SkillId("enemy_skill_fire".into())],
        ultimate: SkillId("enemy_ult_fire".into()),
        follow_up: None,
    }
}

fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain_messages<T: Message + Clone>(cursor: &mut MessageCursor<T>, app: &App) -> Vec<T> {
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// When UltimateCharge.current >= trigger the enemy should emit Ultimate.
#[test]
fn enemy_uses_ultimate_when_charge_ready() {
    let mut app = setup_app();

    app.world_mut().spawn((
        make_unit(1, "Ally", 200),
        Team::Ally,
        Toughness::new(100, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(101, "Enemy", 100),
        Team::Enemy,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 100,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        enemy_skills(),
    ));

    let mut intent_cursor = message_cursor::<ActionIntent>(&mut app);

    app.world_mut().write_message(TurnAdvanced::of(UnitId(101)));
    app.update();

    let intents = drain_messages(&mut intent_cursor, &app);
    assert_eq!(
        intents.len(),
        1,
        "expected exactly one intent, got: {:?}",
        intents
    );
    assert!(
        matches!(
            intents[0],
            ActionIntent::Ultimate {
                attacker: UnitId(101),
                target: UnitId(1),
            }
        ),
        "expected Ultimate(101→1), got {:?}",
        intents[0]
    );
}

/// When UltimateCharge.current < trigger the enemy should fall back to Skill.
#[test]
fn enemy_uses_skill_when_ult_not_ready() {
    let mut app = setup_app();

    app.world_mut().spawn((
        make_unit(1, "Ally", 200),
        Team::Ally,
        Toughness::new(100, vec![]),
    ));
    app.world_mut().spawn((
        make_unit(101, "Enemy", 100),
        Team::Enemy,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 100,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        enemy_skills(),
    ));

    let mut intent_cursor = message_cursor::<ActionIntent>(&mut app);

    app.world_mut().write_message(TurnAdvanced::of(UnitId(101)));
    app.update();

    let intents = drain_messages(&mut intent_cursor, &app);
    assert_eq!(
        intents.len(),
        1,
        "expected exactly one intent, got: {:?}",
        intents
    );
    assert!(
        matches!(
            &intents[0],
            ActionIntent::Skill {
                attacker: UnitId(101),
                skill_id,
                target: UnitId(1),
            } if skill_id.0 == "enemy_skill_fire"
        ),
        "expected Skill(enemy_skill_fire, 101→1), got {:?}",
        intents[0]
    );
}

/// The ally with the lowest toughness ratio (current/max) must be selected as
/// the target regardless of UnitId order.  Ally B (id=2) has ratio 5/50=0.1;
/// Ally A (id=1) has ratio 50/50=1.0, so id=2 must be targeted.
#[test]
fn enemy_targets_lowest_toughness_ratio() {
    let mut app = setup_app();

    // Ally A: full toughness → ratio 1.0
    app.world_mut().spawn((
        make_unit(1, "AllyA", 200),
        Team::Ally,
        Toughness::new(50, vec![]),
    ));
    // Ally B: nearly depleted toughness → ratio 0.1
    app.world_mut().spawn((
        make_unit(2, "AllyB", 200),
        Team::Ally,
        Toughness {
            max: 50,
            current: 5,
            weaknesses: vec![],
            broken: false,
            category: Default::default(),
        },
    ));
    // Enemy: ult not ready → picks Skill → target = lowest toughness ratio
    app.world_mut().spawn((
        make_unit(101, "Enemy", 100),
        Team::Enemy,
        Toughness::new(50, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 100,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        enemy_skills(),
    ));

    let mut intent_cursor = message_cursor::<ActionIntent>(&mut app);

    app.world_mut().write_message(TurnAdvanced::of(UnitId(101)));
    app.update();

    let intents = drain_messages(&mut intent_cursor, &app);
    assert_eq!(
        intents.len(),
        1,
        "expected exactly one intent, got: {:?}",
        intents
    );
    let target = match &intents[0] {
        ActionIntent::Skill { target, .. } => *target,
        ActionIntent::Basic { target, .. } => *target,
        ActionIntent::Ultimate { target, .. } => *target,
    };
    assert_eq!(
        target,
        UnitId(2),
        "expected target UnitId(2) (lowest toughness ratio 5/50), got {:?}",
        target
    );
}
