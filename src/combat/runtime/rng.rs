use rand_core::{Rng, SeedableRng};

use crate::combat::rng::{CombatEntropy, combat_seed_from_u64};

/// Deterministic per-cast RNG using the same `bevy_prng` algorithm as combat ECS RNGs.
///
/// Each (global_seed, cast_id, beat, hop, salt) tuple produces an independent,
/// reproducible stream (invariant I1: same input -> same Intent stream).
///
/// # Separation from `combat::rng::CombatRng`
/// `CombatRng` (`bevy_prng::WyRand` via `bevy_rand`) handles legacy global and
/// per-entity ECS rolls (status accuracy, etc.). `CastRng` is cast-scoped and
/// seeded per invocation, enabling full replay with the same PRNG family.
// Used in cfg(test) within this file; public API for future skill-hook callers.
pub struct CastRng(CombatEntropy);

// All methods consumed in cfg(test) within this file; public API surface for future use.
impl CastRng {
    /// Seed from a raw `u64` using the combat seed adapter shared with `CombatRng`.
    pub fn new(seed: u64) -> Self {
        Self(CombatEntropy::from_seed(combat_seed_from_u64(seed)))
    }

    /// Build from cast context parameters.
    ///
    /// Mixes `(global_seed, cast_id, beat, hop, salt)` into a single seed via
    /// multiplicative hashing so every distinct tuple yields a distinct stream.
    pub fn from_params(global_seed: u64, cast_id: u32, beat: u32, hop: u32, salt: u64) -> Self {
        let mixed = global_seed
            ^ (cast_id as u64).wrapping_mul(0xbf58476d1ce4e5b9)
            ^ (beat as u64).wrapping_mul(0x94d049bb133111eb)
            ^ (hop as u64).wrapping_mul(0x9e3779b97f4a7c15)
            ^ salt;
        Self::new(mixed)
    }

    /// Advances the per-cast PRNG and returns the next 64-bit output.
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// Advances the per-cast PRNG and returns the next 32-bit output.
    pub fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    /// Uniform sample in `[0, n)`. Panics if `n == 0`.
    pub fn next_below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "CastRng::next_below: n must be > 0");
        let rejection_zone = u32::MAX - (u32::MAX % n);
        loop {
            let value = self.next_u32();
            if value < rejection_zone {
                return value % n;
            }
        }
    }

    /// Returns `true` with probability `pct / 100`. Saturates at boundaries.
    pub fn roll_pct(&mut self, pct: u32) -> bool {
        if pct == 0 {
            return false;
        }
        if pct >= 100 {
            return true;
        }
        self.next_below(100) < pct
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

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
}
