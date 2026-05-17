use std::collections::{HashSet, VecDeque};

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::combat::{api::CastId, events::CombatEvent, types::UnitId};

/// Typed signal payload for blueprint-specific reactive logic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalPayload {
    Empty,
    Amount(i64),
    UnitTarget(UnitId),
}

/// A reactive signal dispatched to the global `SignalBus`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// Signal targeted at a specific blueprint logic block.
    Blueprint {
        /// The name of the blueprint that "owns" this signal (e.g., "kitsune_grace").
        owner: String,
        /// The specific signal name (e.g., "ultimate_used").
        name: String,
        /// Optional payload associated with the signal.
        payload: SignalPayload,
        /// The cast that triggered this signal.
        cast_id: CastId,
    },
    /// Bridged kernel combat event envelope for passive listeners.
    CombatEvent(CombatEvent),
}

impl Signal {
    /// Resolve the primary target the passive pipeline should treat as the signal's focus.
    ///
    /// Blueprint signals use `SignalPayload::UnitTarget` when available; all other
    /// payloads fall back to the passive owner's unit id.
    pub fn primary_target(&self, fallback: UnitId) -> UnitId {
        match self {
            Signal::Blueprint { payload, .. } => match payload {
                SignalPayload::UnitTarget(unit) => *unit,
                _ => fallback,
            },
            Signal::CombatEvent(event) => event.target,
        }
    }
}

/// Global reactive signal bus.
///
/// In S04 this carries a `VecDeque<Signal>`. Listeners (like `PassiveRunner`)
/// drain the queue each pipeline step and dispatch to registered hooks.
#[derive(Resource, Default)]
pub struct SignalBus {
    queue: VecDeque<Signal>,
}

impl SignalBus {
    /// Push a new signal onto the bus.
    pub fn push(&mut self, sig: Signal) {
        self.queue.push_back(sig);
    }

    /// Drain all pending signals from the bus.
    pub fn drain(&mut self) -> std::collections::vec_deque::Drain<'_, Signal> {
        self.queue.drain(..)
    }
}

/// Registry of known signal (owner, name) pairs.
///
/// Used by the dispatcher (T02) to verify signals before enqueueing them.
/// Debug builds `debug_assert!` on unregistered signals; release builds drop with a warning.
#[derive(Resource, Default)]
pub struct SignalTaxonomy {
    registered: HashSet<(&'static str, &'static str)>,
}

impl SignalTaxonomy {
    /// Register a valid signal name for a given blueprint owner.
    pub fn register(&mut self, owner: &'static str, name: &'static str) {
        self.registered.insert((owner, name));
    }

    /// Check if a signal name is registered for a given blueprint owner.
    pub fn contains(&self, owner: &'static str, name: &'static str) -> bool {
        self.registered.contains(&(owner, name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::events::{ActionIntentKind, CombatEvent, CombatEventKind};

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
        assert_eq!(bus.queue.len(), 0);
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
}
