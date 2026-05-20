//! Drain helpers for messages and queues.
//!
//! Most tests previously wrote a one-off `cursor.read(messages).cloned().collect()`
//! at the call site. These helpers compress that to a single line and lift the
//! cursor lifecycle into the helper.

#![allow(dead_code)]

use bevy::ecs::message::{Message, MessageCursor, Messages};
use bevy::prelude::*;

use bevyrogue::combat::{
    events::CombatEvent,
    runtime::{Intent, IntentQueue},
};

/// Drain every message of type `T` currently in the bus and return them in
/// insertion order. Subsequent calls return only messages emitted after the
/// previous drain.
pub fn drain<T: Message + Clone>(app: &mut App) -> Vec<T> {
    let mut cursor: MessageCursor<T> = app
        .world_mut()
        .resource_mut::<Messages<T>>()
        .get_cursor_current();
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

/// Drain all messages of type `T` from the start (including ones already
/// read by another cursor). Useful for tests that don't keep a persistent
/// cursor across ticks.
pub fn drain_all<T: Message + Clone>(app: &mut App) -> Vec<T> {
    let mut cursor: MessageCursor<T> = app
        .world_mut()
        .resource_mut::<Messages<T>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

/// Convenience alias for the most common case.
pub fn drain_combat_events(app: &mut App) -> Vec<CombatEvent> {
    drain_all::<CombatEvent>(app)
}

/// Drain the pending [`Intent`] queue (FIFO). Empties the queue.
pub fn drain_intents(app: &mut App) -> Vec<Intent> {
    let mut queue = app.world_mut().resource_mut::<IntentQueue>();
    let mut out = Vec::with_capacity(queue.0.len());
    while let Some(intent) = queue.0.pop_front() {
        out.push(intent);
    }
    out
}
