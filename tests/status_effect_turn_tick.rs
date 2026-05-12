use bevy::prelude::*;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    state::CombatState,
    status_effect::{StatusEffect, StatusEffectKind},
    stun::Stunned,
    team::Team,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system},
    types::{Attribute, EvoStage, SkillId, UnitId},
    unit::Unit,
};

fn unit(id: u32, attribute: Attribute, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 100,
        hp_current,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .add_message::<TurnAdvanced>()
        .add_message::<bevyrogue::combat::av::ActionValueUpdated>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, advance_turn_system);
    app
}

fn combat_events(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

fn action_intents(app: &mut App) -> Vec<ActionIntent> {
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<ActionIntent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<ActionIntent>>())
        .cloned()
        .collect()
}

#[test]
fn heated_ticks_and_expires() {
    // v0 skeleton: no DoT — HP unchanged, but tick/expire events fire correctly.
    let mut app = setup_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Vaccine, 5),
            Team::Ally,
            StatusEffect {
                kind: StatusEffectKind::Heated,
                duration_remaining: 1,
            },
        ))
        .id();
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    assert_eq!(app.world().get::<Unit>(entity).unwrap().hp_current, 5);
    assert!(app.world().get::<StatusEffect>(entity).is_none());
    let events = combat_events(&mut app);
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusTick {
            kind: StatusEffectKind::Heated,
            turns_left: 0,
        }
    )));
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusExpired {
            kind: StatusEffectKind::Heated,
        }
    )));
}

#[test]
fn chilled_ticks_and_expires() {
    // v0 skeleton: no speed delta — SpeedModifier unaffected, but tick/expire events fire.
    let mut app = setup_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Vaccine, 100),
            Team::Ally,
            StatusEffect {
                kind: StatusEffectKind::Chilled,
                duration_remaining: 1,
            },
        ))
        .id();
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    assert!(app.world().get::<StatusEffect>(entity).is_none());
    let events = combat_events(&mut app);
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusTick {
            kind: StatusEffectKind::Chilled,
            turns_left: 0,
        }
    )));
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusExpired {
            kind: StatusEffectKind::Chilled,
        }
    )));
}

#[test]
fn paralyzed_does_not_affect_action_in_v0() {
    // v0 skeleton: Paralyzed is no-op; action proceeds normally.
    let mut app = setup_app();
    app.world_mut()
        .resource_mut::<TurnOrder>()
        .seed([UnitId(1), UnitId(2)]);
    app.world_mut().spawn((
        unit(1, Attribute::Virus, 100),
        Team::Enemy,
        UnitSkills {
            basic: SkillId("basic".into()),
            skills: vec![],
            ultimate: SkillId("ult".into()),
            follow_up: None,
        },
        StatusEffect {
            kind: StatusEffectKind::Paralyzed,
            duration_remaining: 1,
        },
    ));
    app.world_mut()
        .spawn((unit(2, Attribute::Vaccine, 100), Team::Ally));
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    let events = combat_events(&mut app);
    assert!(
        events
            .iter()
            .all(|event| !matches!(&event.kind, CombatEventKind::OnActionFailed { .. }))
    );
}

#[test]
fn paralyzed_lifecycle_in_v0() {
    // v0 skeleton: Paralyzed ticks and expires normally; action cancel deferred to S03.
    let mut app = setup_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Virus, 100),
            Team::Enemy,
            StatusEffect {
                kind: StatusEffectKind::Paralyzed,
                duration_remaining: 1,
            },
        ))
        .id();
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    assert!(app.world().get::<StatusEffect>(entity).is_none());
    let events = combat_events(&mut app);
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusTick {
            kind: StatusEffectKind::Paralyzed,
            turns_left: 0,
        }
    )));
    assert!(events.iter().any(|event| matches!(
        &event.kind,
        CombatEventKind::OnStatusExpired {
            kind: StatusEffectKind::Paralyzed,
        }
    )));
}

#[test]
fn stunned_unit_skips_status_tick() {
    let mut app = setup_app();
    let entity = app
        .world_mut()
        .spawn((
            unit(1, Attribute::Vaccine, 42),
            Team::Ally,
            Stunned { turns_left: 1 },
            StatusEffect {
                kind: StatusEffectKind::Heated,
                duration_remaining: 2,
            },
        ))
        .id();
    app.world_mut().write_message(TurnAdvanced::of(UnitId(1)));

    app.update();

    assert_eq!(app.world().get::<Unit>(entity).unwrap().hp_current, 42);
    assert_eq!(
        app.world().get::<StatusEffect>(entity),
        Some(&StatusEffect {
            kind: StatusEffectKind::Heated,
            duration_remaining: 2,
        })
    );
    assert!(app.world().get::<Stunned>(entity).is_none());
}
