//! Relocated from `src/combat/turn_system/resistance.rs` (R003 — no inline `mod tests` in src/).
//! Unit tests for `apply_advance` / `apply_delay` and the `TempoResistance` curve.

use bevyrogue::combat::av::{ActionValue, MAX_AV};
use bevyrogue::combat::turn_system::resistance::{TempoResistance, apply_advance, apply_delay};

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
