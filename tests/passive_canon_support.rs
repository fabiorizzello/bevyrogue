use bevy::ecs::message::{MessageCursor, Messages};
use bevy::prelude::*;

use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::blueprints::{twin_core::TwinCoreState, patamon::HolySupportState};
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::plugin::CombatPlugin;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::{Attribute, EvoStage, UnitId};
use bevyrogue::combat::unit::Unit;

const AGUMON_ID: UnitId = UnitId(1);
const GABUMON_ID: UnitId = UnitId(2);
const RENAMON_ID: UnitId = UnitId(7);
const PATAMON_ID: UnitId = UnitId(9);
const ALLY_DUMMY_ID: UnitId = UnitId(42);
const ENEMY_DUMMY_ID: UnitId = UnitId(99);

const AGUMON_SIGNAL: (&str, &str) = ("agumon", "apply_heated");
const GABUMON_SIGNAL: (&str, &str) = ("gabumon", "apply_chilled");
const PATAMON_SIGNAL: (&str, &str) = ("patamon", "build_holy_support_grace");
const RENAMON_SIGNAL: (&str, &str) = ("renamon", "kitsune_grace");

fn setup_app() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>().add_plugins(CombatPlugin);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, name: &str, team: Team) {
    app.world_mut().spawn((
        Unit {
            id,
            name: name.to_string(),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Child,
        },
        team,
    ));
}

fn write_ult_used(app: &mut App, unit_id: UnitId) {
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::UltimateUsed { unit_id },
        source: unit_id,
        target: unit_id,
        follow_up_depth: 0,
        cast_id: CastId::ROOT,
    });
}

fn read_events(app: &mut App) -> Vec<CombatEvent> {
    let mut cursor: MessageCursor<CombatEvent> = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

#[test]
fn canonical_passive_bootstrap_routes_ally_ult_to_four_blueprint_reactions() {
    let mut app = setup_app();
    spawn_unit(&mut app, AGUMON_ID, "Agumon", Team::Ally);
    spawn_unit(&mut app, GABUMON_ID, "Gabumon", Team::Ally);
    spawn_unit(&mut app, RENAMON_ID, "Renamon", Team::Ally);
    spawn_unit(&mut app, PATAMON_ID, "Patamon", Team::Ally);
    spawn_unit(&mut app, ALLY_DUMMY_ID, "Dummy Ally", Team::Ally);
    spawn_unit(&mut app, ENEMY_DUMMY_ID, "Dummy Enemy", Team::Enemy);

    write_ult_used(&mut app, ALLY_DUMMY_ID);
    app.update();

    let drained: Vec<_> = app
        .world_mut()
        .resource_mut::<bevyrogue::combat::runtime::SignalBus>()
        .drain()
        .collect();
    assert!(
        drained.is_empty(),
        "SignalBus should be empty after passive dispatch, got: {:?}",
        drained
    );

    let events = read_events(&mut app);
    let blueprint_hits: Vec<_> =
        events
            .iter()
            .filter_map(|event| match &event.kind {
                CombatEventKind::OnKernelTransition {
                    transition:
                        bevyrogue::combat::kernel::CombatKernelTransition::Blueprint {
                            owner, name, ..
                        },
                } => Some((owner.as_str(), name.as_str())),
                _ => None,
            })
            .collect();

    assert!(
        blueprint_hits.contains(&AGUMON_SIGNAL),
        "missing agumon passive reaction: {blueprint_hits:?}"
    );
    assert!(
        blueprint_hits.contains(&GABUMON_SIGNAL),
        "missing gabumon passive reaction: {blueprint_hits:?}"
    );
    assert!(
        blueprint_hits.contains(&PATAMON_SIGNAL),
        "missing patamon passive reaction: {blueprint_hits:?}"
    );
    assert!(
        blueprint_hits.contains(&RENAMON_SIGNAL),
        "missing renamon passive reaction: {blueprint_hits:?}"
    );
    assert_eq!(
        blueprint_hits.len(),
        4,
        "expected exactly four passive blueprint reactions, got: {blueprint_hits:?}"
    );

    let twin = app.world().resource::<TwinCoreState>();
    assert_eq!(
        twin.cross_resonance, 0,
        "passive blueprint routing should stay on the signal bus and not bypass TwinCoreHook"
    );

    let holy = app.world().resource::<HolySupportState>();
    assert_eq!(
        holy.grace, 0,
        "passive blueprint routing should stay on the signal bus and not bypass HolySupportHook"
    );

    let state = app
        .world()
        .resource::<bevyrogue::combat::runtime::BlueprintState>();
    assert_eq!(
        state
            .map
            .get(&(AGUMON_ID, "agumon/twin_core/triggered".to_string())),
        Some(&1)
    );
    assert_eq!(
        state
            .map
            .get(&(GABUMON_ID, "gabumon/twin_core/triggered".to_string())),
        Some(&1)
    );
    assert_eq!(
        state
            .map
            .get(&(PATAMON_ID, "patamon/holy_support/triggered".to_string())),
        Some(&1)
    );
    assert_eq!(
        state
            .map
            .get(&(RENAMON_ID, "renamon/kitsune_grace/triggered".to_string())),
        Some(&1)
    );
}

#[test]
fn kitsune_grace_ignores_self_ult() {
    let mut app = setup_app();
    spawn_unit(&mut app, RENAMON_ID, "Renamon", Team::Ally);
    spawn_unit(&mut app, ALLY_DUMMY_ID, "Dummy Ally", Team::Ally);
    spawn_unit(&mut app, ENEMY_DUMMY_ID, "Dummy Enemy", Team::Enemy);

    write_ult_used(&mut app, RENAMON_ID);
    app.update();

    let events = read_events(&mut app);
    let kitsune_event = events.iter().find(|event| {
        matches!(
            &event.kind,
            CombatEventKind::OnKernelTransition {
                transition: bevyrogue::combat::kernel::CombatKernelTransition::Blueprint { owner, name, .. }
            } if owner == "renamon" && name == "kitsune_grace"
        )
    });
    assert!(
        kitsune_event.is_none(),
        "self ult should not emit kitsune_grace blueprint reaction, got: {:?}",
        events
    );

    let state = app
        .world()
        .resource::<bevyrogue::combat::runtime::BlueprintState>();
    assert!(
        !state
            .map
            .contains_key(&(RENAMON_ID, "renamon/kitsune_grace/triggered".to_string()))
    );
}

#[test]
fn kitsune_grace_ignores_enemy_ult() {
    let mut app = setup_app();
    spawn_unit(&mut app, RENAMON_ID, "Renamon", Team::Ally);
    spawn_unit(&mut app, ALLY_DUMMY_ID, "Dummy Ally", Team::Ally);
    spawn_unit(&mut app, ENEMY_DUMMY_ID, "Dummy Enemy", Team::Enemy);

    write_ult_used(&mut app, ENEMY_DUMMY_ID);
    app.update();

    let events = read_events(&mut app);
    let kitsune_event = events.iter().find(|event| {
        matches!(
            &event.kind,
            CombatEventKind::OnKernelTransition {
                transition: bevyrogue::combat::kernel::CombatKernelTransition::Blueprint { owner, name, .. }
            } if owner == "renamon" && name == "kitsune_grace"
        )
    });
    assert!(
        kitsune_event.is_none(),
        "enemy ult should not emit kitsune_grace blueprint reaction, got: {:?}",
        events
    );

    let state = app
        .world()
        .resource::<bevyrogue::combat::runtime::BlueprintState>();
    assert!(
        !state
            .map
            .contains_key(&(RENAMON_ID, "renamon/kitsune_grace/triggered".to_string()))
    );
}
