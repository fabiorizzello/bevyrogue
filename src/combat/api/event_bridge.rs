use bevy::prelude::*;

use crate::combat::{
    api::signal::{Signal, SignalBus, SignalPayload},
    events::{CombatEvent, CombatEventKind},
};

/// Reads `CombatEvent::UltimateUsed` messages and pushes a corresponding
/// `Signal::Blueprint { owner: "kernel", name: "ult_used" }` onto the `SignalBus`.
///
/// Runs after `intent_applier` and before `passive_dispatch_system` so passive
/// hooks can react to ult casts in the same Update tick.
pub fn combat_event_to_signal_system(
    mut events: MessageReader<CombatEvent>,
    mut bus: ResMut<SignalBus>,
) {
    for event in events.read() {
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
        events::CombatEvent,
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
    fn ult_used_event_pushes_kernel_signal() {
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
            1,
            "expected exactly one ult_used signal, got: {:?}",
            captured.0
        );
        match &captured.0[0] {
            Signal::Blueprint { owner, name, payload, cast_id } => {
                assert_eq!(owner, "kernel");
                assert_eq!(name, "ult_used");
                assert_eq!(*payload, SignalPayload::UnitTarget(unit_id));
                assert_eq!(*cast_id, CastId::ROOT);
            }
        }
    }

    #[test]
    fn non_ult_events_produce_no_signal() {
        use crate::combat::events::ActionIntentKind;

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
        assert!(
            captured.0.is_empty(),
            "non-ult event should not push any signal, got: {:?}",
            captured.0
        );
    }

    #[test]
    fn multiple_ult_events_each_produce_a_signal() {
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
        assert_eq!(
            captured.0.len(),
            2,
            "expected two signals for two ult events, got: {:?}",
            captured.0
        );
    }
}
