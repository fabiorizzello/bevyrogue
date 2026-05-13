use bevy::prelude::*;

/// The maximum Action Value a unit can accumulate before taking a turn.
/// This value is chosen to be large enough to allow for granular speed differences
/// and AV manipulation effects, similar to Honkai: Star Rail.
pub const MAX_AV: i32 = 10000;

/// The amount of Action Value a unit gains per point of Speed per "tick" (when AV is updated).
/// This scales a unit's Speed into its AV gain.
pub const AV_PER_SPEED: i32 = 100;

/// Component to store a unit's current Action Value.
/// A lower value means the unit is closer to taking its turn (i.e., closer to MAX_AV).
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ActionValue(pub i32);

impl ActionValue {
    /// Creates a new ActionValue, ensuring it does not exceed MAX_AV.
    pub fn new(value: i32) -> Self {
        ActionValue(value.min(MAX_AV))
    }

    /// Advances the ActionValue.
    /// This method typically adds to the AV, moving the unit closer to MAX_AV.
    pub fn advance(&mut self, amount: i32) {
        self.0 = (self.0 + amount).min(MAX_AV);
    }

    /// Delays the ActionValue.
    /// This method subtracts from the AV, pushing the unit's turn further back.
    pub fn delay(&mut self, amount: i32) {
        self.0 = (self.0 - amount).max(0);
    }

    /// Self-Advances the ActionValue.
    /// This method adds to the AV, pulling the unit's turn forward.
    /// (Conceptually distinct from `advance` for effect clarity, but mechanically similar here)
    pub fn self_advance(&mut self, amount: i32) {
        self.0 = (self.0 + amount).min(MAX_AV);
    }

    /// Checks if the unit's AV has reached the threshold to take a turn.
    pub fn is_ready(&self) -> bool {
        self.0 >= MAX_AV
    }

    /// Resets the AV to 0 after a unit has taken its turn.
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

/// Message to signal that a unit's Action Value has changed.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionValueUpdated {
    pub unit_entity: Entity,
    pub old_value: i32,
    pub new_value: i32,
}
