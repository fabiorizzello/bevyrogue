use bevy::prelude::*;

use crate::combat::{
    events::{CombatEvent, CombatEventKind},
    runtime::signal::{Signal, SignalBus, SignalPayload},
};

/// Mirrors kernel combat events onto the passive signal bus.
///
/// `CombatEvent` envelopes are bridged verbatim so passive listeners can inspect
/// kernel state transitions without reverse-engineering ad hoc owner/name pairs.
/// `UltimateUsed` still emits the legacy `kernel/ult_used` blueprint signal for
/// existing passive hooks.
pub fn combat_event_to_signal_system(
    mut events: MessageReader<CombatEvent>,
    mut bus: ResMut<SignalBus>,
) {
    for event in events.read() {
        bus.push(Signal::CombatEvent(event.clone()));

        if let CombatEventKind::UltimateUsed { unit_id } = &event.kind {
            bus.push(Signal::Blueprint {
                owner: "kernel".to_string(),
                name: "ult_used".to_string(),
                payload: SignalPayload::UnitTarget(*unit_id),
                cast_id: event.cast_id,
            });
        }
    }
}
