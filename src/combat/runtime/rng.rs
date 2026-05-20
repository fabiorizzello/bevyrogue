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
