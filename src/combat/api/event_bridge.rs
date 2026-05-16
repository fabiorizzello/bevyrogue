use bevy::prelude::*;

use crate::combat::{
    api::signal::{Signal, SignalBus, SignalPayload},
    events::{CombatEvent, CombatEventKind},
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        api::intent::CastId,
        events::{ActionIntentKind, CombatEvent},
        types::UnitId,
    };

    /// Collect-and-drain system resource used inside tests.
    #[derive(Resource, Default)]
    struct CapturedSignals(Vec<Signal>);

    fn drain_system(mut bus: ResMut<SignalBus>, mut cap: ResMut<CapturedSignals>) {
        cap.0.extend(bus.drain());
    }

    fn build_app() -> App {
        let mut app = App::new();
        app.add_message::<CombatEvent>()
            .init_resource::<SignalBus>()
            .init_resource::<CapturedSignals>()
            .add_systems(
                Update,
                (combat_event_to_signal_system, drain_system.after(combat_event_to_signal_system)),
            );
        app
    }

    #[test]
    fn ult_used_event_pushes_combat_and_kernel_signal() {
        let mut app = build_app();

        let unit_id = UnitId(42);
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::UltimateUsed { unit_id },
            source: unit_id,
            target: unit_id,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });

        app.update();

        let captured = app.world().resource::<CapturedSignals>();
        assert_eq!(
            captured.0.len(),
            2,
            "expected combat envelope + legacy ult_used signal, got: {:?}",
            captured.0
        );
        match &captured.0[0] {
            Signal::CombatEvent(event) => {
                assert!(matches!(&event.kind, CombatEventKind::UltimateUsed { unit_id: seen } if *seen == unit_id));
            }
            other => panic!("expected combat envelope first, got: {:?}", other),
        }
        match &captured.0[1] {
            Signal::Blueprint { owner, name, payload, cast_id } => {
                assert_eq!(owner, "kernel");
                assert_eq!(name, "ult_used");
                assert_eq!(*payload, SignalPayload::UnitTarget(unit_id));
                assert_eq!(*cast_id, CastId::ROOT);
            }
            other => panic!("expected legacy blueprint signal second, got: {:?}", other),
        }
    }

    #[test]
    fn non_ult_events_produce_only_combat_envelope() {
        let mut app = build_app();

        let unit_id = UnitId(1);
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnActionDeclared {
                intent_kind: ActionIntentKind::Basic,
            },
            source: unit_id,
            target: unit_id,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });

        app.update();

        let captured = app.world().resource::<CapturedSignals>();
        assert_eq!(captured.0.len(), 1, "non-ult event should bridge as combat envelope only, got: {:?}", captured.0);
        assert!(matches!(captured.0[0], Signal::CombatEvent(_)));
    }

    #[test]
    fn multiple_ult_events_each_produce_both_signals() {
        let mut app = build_app();

        let u1 = UnitId(1);
        let u2 = UnitId(2);
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::UltimateUsed { unit_id: u1 },
            source: u1,
            target: u1,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::UltimateUsed { unit_id: u2 },
            source: u2,
            target: u2,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });

        app.update();

        let captured = app.world().resource::<CapturedSignals>();
        assert_eq!(captured.0.len(), 4, "expected two combat envelopes + two blueprint signals, got: {:?}", captured.0);
    }
}
