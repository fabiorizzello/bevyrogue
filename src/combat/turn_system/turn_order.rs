use bevy::prelude::*;
use std::collections::VecDeque;

use super::types::UnitId;

/// Resource to track the current active unit and the next unit in the turn order.
/// This resource will be managed by the Action Value system.
#[derive(Resource, Debug, Default, Clone)]
pub struct TurnOrder {
    pub active_unit: Option<UnitId>,
    pub next_unit: Option<UnitId>,
    /// Compat shim for pre-AV tests: always empty in the AV system.
    // Compat shim field; only referenced in test comments, not actual test code.
    #[allow(dead_code)]
    pub future_preview: Vec<UnitId>,
    /// Compat shim for pre-AV tests: always empty in the AV system.
    // Consumed by tests/ultimate_meter.rs via order.queue.front().
    #[allow(dead_code)]
    pub queue: VecDeque<UnitId>,
}

// seed/insert_out_of_queue consumed by many tests as compat shims.
#[allow(dead_code)]
impl TurnOrder {
    /// No-op in the AV system; seeds were needed only for the old VecDeque order.
    pub fn seed(&mut self, _units: impl IntoIterator<Item = UnitId>) {}

    /// No-op in the AV system; out-of-queue insertion is superseded by AV manipulation.
    pub fn insert_out_of_queue(&mut self, _unit_id: UnitId) {}
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnAdvanced {
    pub unit_id: UnitId,
    /// The AV value that this unit had when taking its turn.
    pub av_at_turn: i32,
    /// The AV change that occurred when taking the turn.
    pub av_change: i32,
}

impl TurnAdvanced {
    /// Convenience constructor for tests and compatibility code; sets AV metadata to zero.
    // Consumed by many integration tests via TurnAdvanced::of(unit_id).
    #[allow(dead_code)]
    pub fn of(unit_id: UnitId) -> Self {
        TurnAdvanced {
            unit_id,
            av_at_turn: 0,
            av_change: 0,
        }
    }
}

