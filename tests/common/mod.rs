//! Shared fixtures for integration tests.
//!
//! Rust integration-test convention: this file is included with `mod common;`
//! in the consuming test binaries. Each binary instantiates its own copy and
//! only sees the items it imports — the `#![allow(dead_code)]` attribute on
//! each submodule silences "unused helper" warnings in binaries that exercise
//! only part of the API.

#![allow(dead_code)]

pub mod actions;
pub mod apply;
pub mod damage_helpers;
pub mod resolution_helpers;
pub mod units;

use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    av::ActionValueUpdated,
    events::CombatEvent,
    follow_up::{
        FollowUpIntent, FollowUpTrace, follow_up_listener_system, form_identity_listener_system,
        resolve_follow_up_action_system,
    },
    log::ActionLog,
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
    team::Team,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system, check_victory_system, resolve_action_system},
    types::UnitId,
    unit::{Ko, Unit},
};
use bevyrogue::data::{SkillBookHandle, skills_ron::SkillBook, units_ron::UnitRoster};

pub fn load_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

pub fn load_skill_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

pub fn build_app(skill_book: SkillBook) -> App {
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(skill_book);
    let mut app = App::new();
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool { current: 5, max: 5 })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(42))
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionValueUpdated>()
        .add_systems(
            Update,
            (
                resolve_action_system,
                follow_up_listener_system,
                form_identity_listener_system,
                resolve_follow_up_action_system,
                advance_turn_system,
                check_victory_system,
            )
                .chain(),
        );
    app
}

pub fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

pub fn drain_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

pub fn live_enemy_ids(app: &mut App) -> Vec<UnitId> {
    let mut q = app.world_mut().query::<(&Unit, &Team, Option<&Ko>)>();
    let mut ids: Vec<UnitId> = q
        .iter(app.world())
        .filter(|(u, t, ko)| **t == Team::Enemy && u.hp_current > 0 && ko.is_none())
        .map(|(u, _, _)| u.id)
        .collect();
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    ids
}

pub fn is_ally_alive(app: &mut App, id: UnitId) -> bool {
    let mut q = app.world_mut().query::<(&Unit, &Team, Option<&Ko>)>();
    q.iter(app.world())
        .any(|(u, t, ko)| u.id == id && *t == Team::Ally && u.hp_current > 0 && ko.is_none())
}
