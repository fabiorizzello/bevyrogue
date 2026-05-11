use crate::combat::av::{ActionValue, MAX_AV};
use bevy::prelude::*;

/// Units cannot be delayed such that their AV drops below `-MIN_ACTION_THRESHOLD_AV`.
/// With MAX_AV = 10_000, a fully-floored unit still needs 25_000 AV gain to act —
/// preventing infinite-delay lock but allowing meaningful delay punishment.
pub const MIN_ACTION_THRESHOLD_AV: i32 = 15_000;

/// Diminishing returns component for repeated Delay effects (e.g. on bosses).
/// Stack 0 (fresh): 100% effective. Stack 1: 50%. Stack 2+: 25%.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub struct TempoResistance {
    /// Number of Delay hits absorbed so far this combat.
    pub hit_count: u32,
}

impl TempoResistance {
    /// Returns the multiplier applied to the next incoming Delay amount.
    pub fn multiplier(&self) -> f64 {
        match self.hit_count {
            0 => 1.0,
            1 => 0.5,
            _ => 0.25,
        }
    }

    /// Records that a Delay was applied, advancing the resistance stack.
    pub fn record_delay_hit(&mut self) {
        self.hit_count = self.hit_count.saturating_add(1);
    }
}

/// Converts a `TurnAdvance` percentage into a raw AV delta, applying resistance
/// for negative (delay) amounts.
///
/// `amount_pct` follows the skill DSL sign convention:
/// - positive = advance (pull forward)
/// - negative = delay  (push back)
///
/// Advances bypass resistance entirely; only delays are attenuated.
pub fn compute_av_change(amount_pct: i32, resistance: Option<&TempoResistance>) -> i32 {
    let raw = amount_pct * MAX_AV / 100;
    if raw < 0 {
        if let Some(r) = resistance {
            return (raw as f64 * r.multiplier()).round() as i32;
        }
    }
    raw
}

/// Applies a `TurnAdvance`-style AV mutation to `av`, respecting resistance and the
/// MIN_ACTION_THRESHOLD_AV floor.
///
/// Returns the actual AV delta applied (negative = unit was delayed, positive = advanced).
/// The resistance stack is incremented only when a delay is applied.
pub fn apply_av_change(
    av: &mut ActionValue,
    resistance: Option<&mut TempoResistance>,
    amount_pct: i32,
) -> i32 {
    let eff = compute_av_change(amount_pct, resistance.as_deref());
    let old = av.0;
    if eff < 0 {
        av.0 = (av.0 + eff).max(-MIN_ACTION_THRESHOLD_AV);
        if let Some(r) = resistance {
            r.record_delay_hit();
        }
    } else {
        av.0 = (av.0 + eff).min(MAX_AV);
    }
    av.0 - old
}
