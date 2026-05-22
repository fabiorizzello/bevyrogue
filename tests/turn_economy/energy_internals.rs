//! Relocated from `src/combat/mechanics/energy.rs` (R003 — no inline `mod tests` in src/).
//! Unit tests for `Energy` gain clamping. The only ceiling on energy gain is
//! `Energy.max`; there is no per-round source cap.

use bevyrogue::combat::mechanics::energy::Energy;

#[test]
fn energy_gain_clamps_at_max() {
    let mut e = Energy::default();
    e.gain(150);
    assert_eq!(e.current, e.max);
    assert_eq!(e.current, 100);
}

#[test]
fn repeated_gains_accumulate_without_round_cap() {
    // Previously a per-round "secondary" cap stopped accumulation at 10. With the
    // cap removed, repeated grants accumulate freely until they reach `max`.
    let mut e = Energy::default();
    e.gain(6);
    e.gain(6);
    e.gain(6);
    assert_eq!(e.current, 18, "gains accumulate past the old per-round cap of 10");
}

#[test]
fn gain_capped_reports_only_the_amount_actually_added() {
    let mut e = Energy { current: 95, max: 100 };
    let applied = e.gain_capped(20);
    assert_eq!(applied, 5, "only the headroom up to max is applied");
    assert_eq!(e.current, 100);
}
