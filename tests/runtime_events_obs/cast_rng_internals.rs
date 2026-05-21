//! Relocated from `src/combat/runtime/rng.rs` (R003 — no inline `mod tests` in src/).
//! Unit tests for `CastRng` — determinism, parameter sensitivity, and distribution.

use bevyrogue::combat::runtime::rng::CastRng;

#[test]
fn determinism_same_seed() {
    let mut a = CastRng::new(12345);
    let mut b = CastRng::new(12345);
    for _ in 0..20 {
        assert_eq!(
            a.next_u64(),
            b.next_u64(),
            "same seed must produce same sequence"
        );
    }
}

#[test]
fn different_seeds_diverge() {
    let seq = |seed: u64| -> Vec<u64> {
        let mut r = CastRng::new(seed);
        (0..8).map(|_| r.next_u64()).collect()
    };
    assert_ne!(seq(1), seq(2));
    assert_ne!(seq(0), seq(u64::MAX));
}

#[test]
fn from_params_determinism() {
    let seq = |mut r: CastRng| -> Vec<u64> { (0..8).map(|_| r.next_u64()).collect() };
    assert_eq!(
        seq(CastRng::from_params(999, 1, 2, 3, 42)),
        seq(CastRng::from_params(999, 1, 2, 3, 42))
    );
}

#[test]
fn from_params_sensitivity() {
    // Changing any single parameter must produce a different stream.
    let base_seq = |r: &mut CastRng| -> Vec<u64> { (0..8).map(|_| r.next_u64()).collect() };
    let base = base_seq(&mut CastRng::from_params(0, 0, 0, 0, 0));
    let diff_seed = base_seq(&mut CastRng::from_params(1, 0, 0, 0, 0));
    let diff_cast = base_seq(&mut CastRng::from_params(0, 1, 0, 0, 0));
    let diff_beat = base_seq(&mut CastRng::from_params(0, 0, 1, 0, 0));
    let diff_hop = base_seq(&mut CastRng::from_params(0, 0, 0, 1, 0));
    let diff_salt = base_seq(&mut CastRng::from_params(0, 0, 0, 0, 1));
    assert_ne!(base, diff_seed);
    assert_ne!(base, diff_cast);
    assert_ne!(base, diff_beat);
    assert_ne!(base, diff_hop);
    assert_ne!(base, diff_salt);
}

#[test]
fn roll_pct_boundaries() {
    let mut r = CastRng::new(0);
    assert!(!r.roll_pct(0), "0% must always be false");
    assert!(r.roll_pct(100), "100% must always be true");
    assert!(r.roll_pct(101), ">100% must always be true");
}

#[test]
fn next_below_distribution_coarse() {
    // All values in [0, 6) must appear in 1000 draws (collision resistance).
    let mut r = CastRng::new(777);
    let mut seen = [false; 6];
    for _ in 0..1000 {
        let v = r.next_below(6);
        assert!((v as usize) < 6, "value out of range");
        seen[v as usize] = true;
    }
    assert!(seen.iter().all(|&s| s), "all values 0..6 must appear");
}
