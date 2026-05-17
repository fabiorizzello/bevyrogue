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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::av::{ActionValue, MAX_AV};

    // ── apply_advance ─────────────────────────────────────────────────────

    #[test]
    fn advance_cap_50_enforced() {
        let mut av = ActionValue(0);
        let delta = apply_advance(&mut av, 80);
        // capped 80 → 50; 50 * 10_000 / 100 = 5_000
        assert_eq!(delta, 5_000);
        assert_eq!(av.0, 5_000);
    }

    #[test]
    fn advance_ceiling_2x_max_av() {
        let mut av = ActionValue(MAX_AV); // 10_000
        let delta = apply_advance(&mut av, 50);
        assert_eq!(av.0, 15_000);
        assert_eq!(delta, 5_000);
    }

    #[test]
    fn double_advance_clamps_at_2x_ceiling() {
        let mut av = ActionValue(MAX_AV); // 10_000
        apply_advance(&mut av, 50); // → 15_000
        apply_advance(&mut av, 50); // → 20_000
        assert_eq!(av.0, 2 * MAX_AV, "ceiling 20_000");
        let delta = apply_advance(&mut av, 50); // already at ceiling
        assert_eq!(av.0, 2 * MAX_AV, "stays at ceiling");
        assert_eq!(delta, 0);
    }

    // ── apply_delay ──────────────────────────────────────────────────────

    #[test]
    fn delay_cap_50_enforced() {
        let mut av = ActionValue(MAX_AV); // 10_000
        let delta = apply_delay(&mut av, 80, None);
        // capped 80 → 50; 50 * 10_000 / 100 = 5_000; 10_000 - 5_000 = 5_000
        assert_eq!(av.0, 5_000);
        assert_eq!(delta, -5_000);
    }

    #[test]
    fn delay_floor_0_no_negative_av() {
        let mut av = ActionValue(2_000);
        let delta = apply_delay(&mut av, 50, None);
        // 50 * 10_000 / 100 = 5_000; 2_000 - 5_000 → clamped to 0
        assert_eq!(av.0, 0);
        assert_eq!(delta, -2_000, "delta clamped to available headroom");
    }

    #[test]
    fn delay_with_resistance_full_curve() {
        let mut av = ActionValue(MAX_AV); // 10_000
        let mut r = TempoResistance::default();
        apply_delay(&mut av, 20, Some(&mut r)); // 100%: -2_000 → 8_000
        apply_delay(&mut av, 20, Some(&mut r)); // 50%:  -1_000 → 7_000
        apply_delay(&mut av, 20, Some(&mut r)); // 25%:  -500   → 6_500
        assert_eq!(av.0, 6_500);
        assert_eq!(r.hit_count, 3);
    }

    #[test]
    fn delay_records_hit_and_updates_av() {
        let mut av = ActionValue(MAX_AV / 2); // 5_000
        let mut r = TempoResistance::default();
        let delta = apply_delay(&mut av, 30, Some(&mut r));
        // 30 ≤ 50 → raw 3_000; 100% → 3_000; 5_000 - 3_000 = 2_000
        assert_eq!(av.0, 2_000);
        assert_eq!(delta, -3_000);
        assert_eq!(r.hit_count, 1);
    }

    #[test]
    fn advance_bypasses_resistance_semantically() {
        // apply_advance has no resistance param — advance always full strength
        let mut av = ActionValue(0);
        let delta = apply_advance(&mut av, 30);
        assert_eq!(delta, 3_000);
    }
}
