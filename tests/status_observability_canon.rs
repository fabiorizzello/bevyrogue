/// Integration test: §H.1 canon status vocabulary in JSONL event stream (S06/T02).
///
/// Scenario: 5 ally units, one per canon kind (Heated/Chilled/Paralyzed/Slowed/Blessed),
/// statuses applied directly via StatusBag::apply(dur=2). Two rounds of TurnAdvanced drive
/// OnStatusTick + OnStatusExpired for all kinds.
///
/// Assertions:
///   (a) Each of the 5 canon kind names appears as "kind":"<Name>" in the serialized stream.
///   (b) Zero matches for "kind":"Freeze", "kind":"DeepFreeze", "kind":"Burn", "kind":"Shock".
///       Anchored on "kind":"…" to avoid false-positives from damage_tag:"Fire" etc.
///   (c) ValidationSnapshot.statuses matches expected hand-rolled vectors per unit
///       (captured after round 1, before statuses expire).
use bevy::ecs::message::Messages;
use bevy::prelude::*;
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    av::ActionValueUpdated,
    blueprints::agumon::TwinCoreState,
    events::CombatEvent,
    log::ActionLog,
    observability::{ValidationStatusSnapshot, capture_validation_snapshot},
    rng::CombatRng,
    sp::SpPool,
    state::CombatState,
    team::Team,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system},
    types::{Attribute, EvoStage, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};

fn make_unit(id: u32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 500,
        hp_current: 500,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn default_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

#[test]
fn canon_status_vocab_in_jsonl_stream_and_validation_snapshot() {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .init_resource::<TwinCoreState>()
        .init_resource::<SpPool>()
        .insert_resource(CombatRng::from_seed(0))
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, advance_turn_system);

    // 5 ally units, one per canon status (dur=2 survives round 1 for snapshot assertion).
    let unit_statuses: [(u32, StatusEffectKind); 5] = [
        (1, StatusEffectKind::Heated),
        (2, StatusEffectKind::Chilled),
        (3, StatusEffectKind::Paralyzed),
        (4, StatusEffectKind::Slowed),
        (5, StatusEffectKind::Blessed),
    ];

    for (id, kind) in &unit_statuses {
        let mut bag = StatusBag::default();
        bag.apply(kind.clone(), 2);
        app.world_mut()
            .spawn((make_unit(*id), Team::Ally, bag, default_ult()));
    }

    // Cursor initialized at current write-head — reads only events emitted after this point.
    // IMPORTANT: Messages<T> double-buffers; events older than 2 frames are cleared.
    // Must drain cursor every frame to avoid losing events from earlier frames.
    let mut event_cursor = app
        .world()
        .resource::<Messages<CombatEvent>>()
        .get_cursor_current();

    let mut all_event_strings: Vec<String> = Vec::new();

    // Round 1: one TurnAdvanced per unit → OnStatusTick emitted, dur decrements to 1.
    // Drain cursor each frame before events are cleared by the double-buffer swap.
    for (id, _) in &unit_statuses {
        app.world_mut().write_message(TurnAdvanced::of(UnitId(*id)));
        app.update();

        let frame_strings: Vec<String> = {
            let msgs = app.world().resource::<Messages<CombatEvent>>();
            event_cursor
                .read(msgs)
                .map(|ev| serde_json::to_string(ev).unwrap())
                .collect()
        };
        all_event_strings.extend(frame_strings);
    }

    // (c) Snapshot after round 1: each unit must carry its status with duration_remaining=1.
    let snapshot =
        capture_validation_snapshot(app.world_mut()).expect("snapshot must build cleanly");

    for (id, kind) in &unit_statuses {
        let unit_snap = snapshot
            .units
            .iter()
            .find(|u| u.id == UnitId(*id))
            .unwrap_or_else(|| panic!("unit {id} missing from snapshot"));
        assert_eq!(
            unit_snap.statuses,
            vec![ValidationStatusSnapshot {
                kind: kind.clone(),
                duration_remaining: 1
            }],
            "unit {id} ({kind:?}): unexpected statuses in snapshot"
        );
    }

    // Round 2: one more TurnAdvanced per unit → OnStatusTick(turns_left=0) + OnStatusExpired.
    for (id, _) in &unit_statuses {
        app.world_mut().write_message(TurnAdvanced::of(UnitId(*id)));
        app.update();

        let frame_strings: Vec<String> = {
            let msgs = app.world().resource::<Messages<CombatEvent>>();
            event_cursor
                .read(msgs)
                .map(|ev| serde_json::to_string(ev).unwrap())
                .collect()
        };
        all_event_strings.extend(frame_strings);
    }

    // Join all per-frame event strings for vocabulary assertions.
    let stream = all_event_strings.join("\n");

    assert!(!stream.is_empty(), "event stream must not be empty");

    // (a) Each canon kind name must appear as "kind":"<Name>" in the stream.
    for name in ["Heated", "Chilled", "Paralyzed", "Slowed", "Blessed"] {
        let needle = format!("\"kind\":\"{}\"", name);
        assert!(
            stream.contains(&needle),
            "canon kind \"{name}\" absent from event stream;\nstream:\n{stream}"
        );
    }

    // (b) No legacy or reserved variant names (anchored on "kind":"…" to avoid
    //     false-positives such as "damage_tag":"Fire").
    for name in ["Freeze", "DeepFreeze", "Burn", "Shock"] {
        let needle = format!("\"kind\":\"{}\"", name);
        assert!(
            !stream.contains(&needle),
            "legacy/reserved kind \"{name}\" leaked into event stream"
        );
    }
}
