use std::collections::{HashSet, VecDeque};

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::combat::{runtime::CastId, events::CombatEvent, types::UnitId};

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

