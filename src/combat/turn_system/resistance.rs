use crate::combat::av::{ActionValue, MAX_AV};
use bevy::prelude::*;

/// Per-call cap on any time-manipulation percentage (advance or delay).
/// Prevents single-skill AV swings beyond ±50% of MAX_AV.
pub const CAP_PCT: u32 = 50;

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

/// Advances `av` by `pct` percent of MAX_AV.
///
/// `pct` is defensively capped at [`CAP_PCT`] (50) before computation.
/// AV is clamped to `[0, 2*MAX_AV]` after addition.
/// Returns the actual AV delta (non-negative).
pub fn apply_advance(av: &mut ActionValue, pct: u32) -> i32 {
    let capped = pct.min(CAP_PCT);
    let raw = (capped as i32) * MAX_AV / 100;
    let old = av.0;
    av.0 = (av.0 + raw).clamp(0, 2 * MAX_AV);
    av.0 - old
}

/// Delays `av` by `pct` percent of MAX_AV, with optional `TempoResistance` attenuation.
///
/// `pct` is defensively capped at [`CAP_PCT`] (50) before computation.
/// AV is clamped to `[0, 2*MAX_AV]` after addition.
/// Returns the actual AV delta (non-positive).
pub fn apply_delay(
    av: &mut ActionValue,
    pct: u32,
    resistance: Option<&mut TempoResistance>,
) -> i32 {
    let capped = pct.min(CAP_PCT);
    let raw = (capped as i32) * MAX_AV / 100;
    let eff = match resistance.as_deref() {
        Some(r) => (raw as f64 * r.multiplier()).round() as i32,
        None => raw,
    };
    let old = av.0;
    av.0 = (av.0 - eff).clamp(0, 2 * MAX_AV);
    if let Some(r) = resistance {
        r.record_delay_hit();
    }
    av.0 - old
}
