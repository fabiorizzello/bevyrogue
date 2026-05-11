use bevy::prelude::Component;

/// Per-unit flags reset each round. Tracks Break Seal and Form Identity usage.
#[derive(Component, Debug, Clone, Default)]
pub struct RoundFlags {
    /// When true, `Toughness::apply_hit` short-circuits to false — prevents repeated breaks
    /// on the same defender within a single round.
    pub break_sealed: bool,
    /// When true, form identity has already fired this round for this unit.
    /// Reset to false at the start of each new turn (advance_turn_system).
    pub form_identity_used: bool,
    /// Total hits received by this unit during the current round.
    pub hits_received_this_round: u32,
    /// True if the unit performed an action during its turn in the current round.
    pub acted_this_turn: bool,
    /// True if the unit performed an action during its turn in the previous round.
    pub acted_last_turn: bool,
}
