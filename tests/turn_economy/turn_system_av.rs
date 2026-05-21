
use bevy::prelude::*;
use bevyrogue::combat::av::{ActionValue, MAX_AV};
use bevyrogue::combat::speed::Speed;
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::stun::Stunned;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::turn_order::TurnOrder;
use bevyrogue::combat::turn_system::advance_turn_system;
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Ko;
use bevyrogue::combat::unit::Unit;

use crate::common::app::turn_av_base_app;

fn uid(n: u32) -> UnitId {
    UnitId(n)
}

fn setup_app() -> App {
    let mut app = turn_av_base_app();
    app.add_systems(Update, advance_turn_system);
    app
}

fn spawn_unit(world: &mut World, id: UnitId, speed: i32, team: Team) -> Entity {
    world
        .spawn((
            Unit {
                id,
                name: format!("Unit_{}", id.0),
                hp_max: 100,
                hp_current: 100,
                attribute: bevyrogue::combat::types::Attribute::Free,
                resists: vec![],
                evo_stage: bevyrogue::combat::types::EvoStage::Child,
            },
            Speed(speed),
            ActionValue(0),
            team,
            bevyrogue::combat::speed::SpeedModifier(0),
        ))
        .id()
}

#[test]
fn av_basic_advancement_and_turn() {
    let mut app = setup_app();

    let entity1 = spawn_unit(&mut app.world_mut(), uid(1), 100, Team::Ally);
    let entity2 = spawn_unit(&mut app.world_mut(), uid(2), 50, Team::Ally);

    app.world_mut().resource_mut::<CombatState>().phase = CombatPhase::WaitingForTurn;

    // With speed=100 and AV_PER_SPEED=100, unit1 gains 10000 AV per tick = MAX_AV in one tick
    for _ in 0..10 {
        app.update();
        let turn_order = app.world().resource::<TurnOrder>();
        if turn_order.active_unit.is_some() {
            break;
        }
    }

    let turn_order = app.world().resource::<TurnOrder>();
    assert_eq!(turn_order.active_unit, Some(uid(1)));
    assert_eq!(app.world().get::<ActionValue>(entity1).unwrap().0, 0); // AV reset for active unit
    assert!(app.world().get::<ActionValue>(entity2).unwrap().0 > 0); // unit2 accumulated some AV

    assert_eq!(
        app.world().resource::<CombatState>().phase,
        CombatPhase::WaitingAction
    );

    // Simulate unit taking its action, then reset phase to continue
    app.world_mut().resource_mut::<CombatState>().phase = CombatPhase::WaitingForTurn;
    app.update();

    // Unit1 should have re-accumulated AV (speed=100, one tick = 10000 = MAX_AV)
    assert!(app.world().get::<ActionValue>(entity1).unwrap().0 > 0);
}

#[test]
fn av_tie_breaking_by_unit_id() {
    let mut app = setup_app();

    let entity1 = spawn_unit(&mut app.world_mut(), uid(1), 100, Team::Ally);
    let entity2 = spawn_unit(&mut app.world_mut(), uid(2), 100, Team::Ally); // Same speed

    app.world_mut().resource_mut::<CombatState>().phase = CombatPhase::WaitingForTurn;

    for _ in 0..10 {
        app.update();
        let turn_order = app.world().resource::<TurnOrder>();
        if turn_order.active_unit.is_some() {
            break;
        }
    }

    let turn_order = app.world().resource::<TurnOrder>();
    // Unit with lower UnitId should go first
    assert_eq!(turn_order.active_unit, Some(uid(1)));
    assert_eq!(app.world().get::<ActionValue>(entity1).unwrap().0, 0);
    // entity2 also reached MAX_AV in same tick, so it's reset to 0 too? No - only the winner is reset.
    // entity2 AV should still be MAX_AV since only entity1 was selected and reset.
    assert_eq!(app.world().get::<ActionValue>(entity2).unwrap().0, MAX_AV);
}

#[test]
fn av_stunned_unit_does_not_advance() {
    let mut app = setup_app();

    let entity1 = spawn_unit(&mut app.world_mut(), uid(1), 100, Team::Ally);
    let entity2 = spawn_unit(&mut app.world_mut(), uid(2), 100, Team::Ally);

    // Stun entity1 — Without<Stunned> filter keeps it out of AV advancement
    app.world_mut()
        .entity_mut(entity1)
        .insert(Stunned { turns_left: 1 });

    app.world_mut().resource_mut::<CombatState>().phase = CombatPhase::WaitingForTurn;

    for _ in 0..10 {
        app.update();
        let turn_order = app.world().resource::<TurnOrder>();
        if turn_order.active_unit.is_some() {
            break;
        }
    }

    let turn_order = app.world().resource::<TurnOrder>();
    assert_eq!(turn_order.active_unit, Some(uid(2)));
    assert_eq!(app.world().get::<ActionValue>(entity2).unwrap().0, 0);
    assert_eq!(app.world().get::<ActionValue>(entity1).unwrap().0, 0); // Stunned unit never advanced
}

#[test]
fn av_ko_unit_does_not_advance() {
    let mut app = setup_app();

    let entity1 = spawn_unit(&mut app.world_mut(), uid(1), 100, Team::Ally);
    let entity2 = spawn_unit(&mut app.world_mut(), uid(2), 100, Team::Ally);

    // KO entity1 — Without<Ko> filter keeps it out of AV advancement
    app.world_mut().entity_mut(entity1).insert(Ko);

    app.world_mut().resource_mut::<CombatState>().phase = CombatPhase::WaitingForTurn;

    for _ in 0..10 {
        app.update();
        let turn_order = app.world().resource::<TurnOrder>();
        if turn_order.active_unit.is_some() {
            break;
        }
    }

    let turn_order = app.world().resource::<TurnOrder>();
    assert_eq!(turn_order.active_unit, Some(uid(2)));
    assert_eq!(app.world().get::<ActionValue>(entity2).unwrap().0, 0);
    assert_eq!(app.world().get::<ActionValue>(entity1).unwrap().0, 0); // KO'd unit never advanced
}
