use bevy::{ecs::message::MessageCursor, ecs::system::RunSystemOnce, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    runtime::{
        CastId, Intent, Signal, SignalBus, SignalPayload, SignalTaxonomy,
        applier::{IntentQueue, intent_applier},
        blueprint_state::BlueprintState,
    },
    types::UnitId,
};

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<IntentQueue>()
        .init_resource::<SignalBus>()
        .init_resource::<SignalTaxonomy>()
        .init_resource::<BlueprintState>()
        .add_message::<CombatEvent>();
    app
}

fn drain_events(app: &mut App) -> Vec<CombatEvent> {
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
fn test_blueprint_signal_dispatching() {
    let mut app = setup_app();

    // Register signal
    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register("test", "sig");

    let source = UnitId(0);
    let cast_id = CastId::ROOT;
    let payload = SignalPayload::Amount(42);

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::BlueprintSignal {
            source,
            owner: "test",
            name: "sig",
            payload: payload.clone(),
            cast_id,
        });

    app.world_mut()
        .run_system_once(intent_applier)
        .expect("intent_applier system runs");

    // Assert SignalBus
    let signals: Vec<_> = app
        .world_mut()
        .resource_mut::<SignalBus>()
        .drain()
        .collect();
    assert_eq!(signals.len(), 1);
    match &signals[0] {
        Signal::Blueprint {
            owner,
            name,
            payload: sig_payload,
            ..
        } => {
            assert_eq!(*owner, "test");
            assert_eq!(*name, "sig");
            assert_eq!(*sig_payload, payload);
        }
        Signal::CombatEvent(event) => panic!(
            "expected blueprint signal, got combat envelope: {:?}",
            event
        ),
    }

    // Assert CombatEvent
    let events = drain_events(&mut app);
    let blueprint_ev = events.iter().find(|e| {
        matches!(
            e.kind,
            CombatEventKind::OnKernelTransition {
                transition: CombatKernelTransition::Blueprint { .. }
            }
        )
    });
    assert!(
        blueprint_ev.is_some(),
        "Expected OnKernelTransition::Blueprint event, got {:?}",
        events
    );
    let ev = blueprint_ev.unwrap();
    if let CombatEventKind::OnKernelTransition {
        transition:
            CombatKernelTransition::Blueprint {
                owner,
                name,
                payload: ev_payload,
            },
    } = &ev.kind
    {
        assert_eq!(*owner, "test");
        assert_eq!(*name, "sig");
        assert_eq!(*ev_payload, payload);
    }
}

#[test]
fn test_set_blueprint_state() {
    let mut app = setup_app();
    let actor = UnitId(123);
    let key = "kitsune_grace/stacks".to_string();
    let value = 5;

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::SetBlueprintState {
            actor,
            key: key.clone(),
            value,
            cast_id: CastId::ROOT,
        });

    app.world_mut()
        .run_system_once(intent_applier)
        .expect("intent_applier system runs");

    let state = app.world().resource::<BlueprintState>();
    assert_eq!(state.map.get(&(actor, key)), Some(&value));
}

#[test]
#[should_panic(expected = "unregistered signal: test/unregistered")]
fn test_unregistered_signal_panics_in_debug() {
    let mut app = setup_app();

    app.world_mut()
        .resource_mut::<IntentQueue>()
        .0
        .push_back(Intent::BlueprintSignal {
            source: UnitId(0),
            owner: "test",
            name: "unregistered",
            payload: SignalPayload::Empty,
            cast_id: CastId::ROOT,
        });

    app.world_mut()
        .run_system_once(intent_applier)
        .expect("intent_applier system runs");
}
