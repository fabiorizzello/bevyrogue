use bevy::prelude::*;

use crate::combat::types::UnitId;

/// Tracks the unit currently entitled to act. Turn ordering itself is owned by
/// the Action Value system (`advance_turn_system`), which writes `active_unit`
/// when a unit's AV crosses the ready threshold.
#[derive(Resource, Debug, Default, Clone)]
pub struct TurnOrder {
    pub active_unit: Option<UnitId>,
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
    pub fn of(unit_id: UnitId) -> Self {
        TurnAdvanced {
            unit_id,
            av_at_turn: 0,
            av_change: 0,
        }
    }
}

