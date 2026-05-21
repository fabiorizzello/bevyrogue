//! Relocated from `src/combat/runtime/signal.rs` (R003 — no inline `mod tests` in src/).
//! Unit tests for SignalBus drain order, SignalTaxonomy registration, and Signal serde
//! round-trip (payload + CombatEvent variants).
//!
//! Note: the original inline test asserted `bus.queue.len() == 0` post-drain to
//! verify drain empties the queue. `queue` is a private field; this relocation
//! replaces that with the equivalent behavioural assertion — a second drain
//! must yield zero elements.

use bevyrogue::combat::events::{ActionIntentKind, CombatEvent, CombatEventKind};
use bevyrogue::combat::runtime::intent::CastId;
use bevyrogue::combat::runtime::signal::{Signal, SignalBus, SignalPayload, SignalTaxonomy};
use bevyrogue::combat::types::UnitId;

#[test]
fn test_push_drain_order() {
    let mut bus = SignalBus::default();
    let sig1 = Signal::Blueprint {
        owner: "o1".to_string(),
        name: "n1".to_string(),
        payload: SignalPayload::Empty,
        cast_id: CastId::ROOT,
    };
    let sig2 = Signal::Blueprint {
        owner: "o2".to_string(),
        name: "n2".to_string(),
        payload: SignalPayload::Amount(42),
        cast_id: CastId::ROOT,
    };

    bus.push(sig1.clone());
    bus.push(sig2.clone());

    let drained: Vec<_> = bus.drain().collect();
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0], sig1);
    assert_eq!(drained[1], sig2);
    // Behavioural check (replaces `bus.queue.len() == 0`): second drain is empty.
    assert_eq!(bus.drain().count(), 0, "drain must empty the bus");
}

#[test]
fn test_taxonomy_register_contains() {
    let mut tax = SignalTaxonomy::default();
    assert!(!tax.contains("owner", "name"));

    tax.register("owner", "name");
    assert!(tax.contains("owner", "name"));
    assert!(!tax.contains("owner", "other"));
}

#[test]
fn test_signal_payload_round_trip() {
    let p1 = SignalPayload::Empty;
    let p2 = SignalPayload::Amount(-123);
    let p3 = SignalPayload::UnitTarget(UnitId(1));

    let s1 = serde_json::to_string(&p1).unwrap();
    let s2 = serde_json::to_string(&p2).unwrap();
    let s3 = serde_json::to_string(&p3).unwrap();

    assert_eq!(serde_json::from_str::<SignalPayload>(&s1).unwrap(), p1);
    assert_eq!(serde_json::from_str::<SignalPayload>(&s2).unwrap(), p2);
    assert_eq!(serde_json::from_str::<SignalPayload>(&s3).unwrap(), p3);
}

#[test]
fn test_combat_event_round_trip() {
    let sig = Signal::CombatEvent(CombatEvent {
        kind: CombatEventKind::OnActionDeclared {
            intent_kind: ActionIntentKind::Skill,
        },
        source: UnitId(2),
        target: UnitId(3),
        follow_up_depth: 1,
        cast_id: CastId::ROOT,
    });

    let json = serde_json::to_string(&sig).unwrap();
    let round_tripped: Signal = serde_json::from_str(&json).unwrap();
    assert_eq!(round_tripped, sig);
}
