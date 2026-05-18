/// Integration test for §H.1 Paralyzed semantics (S04/T01): always-skip across full duration.
///
/// Scenario: single enemy unit with Paralyzed(dur=100), 100 TurnAdvanced cycles.
/// Asserts:
///   - exactly 100 OnActionFailed{reason:"paralyzed"} events (one per turn, including last tick)
///   - zero ActionIntent from the enemy across all 100 turns
use bevy::prelude::*;
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    av::ActionValueUpdated,
    events::{CombatEvent, CombatEventKind},
    log::ActionLog,
    rng::CombatRng,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system},
    types::{Attribute, EvoStage, UnitId},
    unit::Unit,
};

fn setup_app() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(0))
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, advance_turn_system);

    let ally = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(1),
                name: "Ally".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            Toughness::new(100, vec![]),
            StatusBag::default(),
        ))
        .id();

    let enemy = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Enemy".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness::new(100, vec![]),
            {
                let mut bag = StatusBag::default();
                bag.apply(StatusEffectKind::Paralyzed, 100);
                bag
            },
        ))
        .id();

    (app, ally, enemy)
}

#[test]
fn paralyzed_enemy_skips_all_100_turns() {
    let (mut app, _ally, enemy) = setup_app();
    let enemy_id = app.world().get::<Unit>(enemy).unwrap().id;

    // Create cursors at current write-head so each read() returns only that frame's new messages.
    let mut event_cursor = app
        .world()
        .resource::<Messages<CombatEvent>>()
        .get_cursor_current();
    let mut intent_cursor = app
        .world()
        .resource::<Messages<ActionIntent>>()
        .get_cursor_current();

    let mut skip_count = 0usize;
    let mut enemy_intent_count = 0usize;

    for _ in 0..100 {
        app.world_mut().write_message(TurnAdvanced::of(enemy_id));
        app.update();

        let frame_events: Vec<CombatEvent> = {
            let msgs = app.world().resource::<Messages<CombatEvent>>();
            event_cursor.read(msgs).cloned().collect()
        };
        for ev in &frame_events {
            if matches!(&ev.kind, CombatEventKind::OnActionFailed { reason } if reason == "paralyzed")
            {
                skip_count += 1;
            }
        }

        let frame_intents: Vec<ActionIntent> = {
            let msgs = app.world().resource::<Messages<ActionIntent>>();
            intent_cursor.read(msgs).cloned().collect()
        };
        for intent in &frame_intents {
            if let ActionIntent::Skill { attacker, .. } = intent {
                if *attacker == enemy_id {
                    enemy_intent_count += 1;
                }
            }
        }
    }

    assert_eq!(
        skip_count, 100,
        "expected 100 OnActionFailed{{reason:\"paralyzed\"}} across 100 turns; got {skip_count}"
    );
    assert_eq!(
        enemy_intent_count, 0,
        "paralyzed enemy must dispatch zero ActionIntents across 100 turns; got {enemy_intent_count}"
    );
}
