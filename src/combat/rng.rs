use bevy::prelude::Resource;
use rand::{Rng, SeedableRng, rngs::StdRng};

/// Centralised combat RNG resource (R019).
///
/// Wraps a seeded `StdRng` so every random decision in combat (status accuracy
/// rolls, Shock cancel rolls) goes through one inspectable, re-seedable source.
/// Use `CombatRng::from_seed` in tests for deterministic outcomes.
#[derive(Resource)]
pub struct CombatRng(StdRng);

impl CombatRng {
    /// Seed from a single `u64` (uses `SeedableRng::seed_from_u64` internally).
    pub fn from_seed(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }

    /// Returns `true` if a uniform draw in `[0, 100)` is strictly less than
    /// `threshold`.  Boundary cases are clamped:
    ///   - `threshold ≤ 0` → always `false`
    ///   - `threshold ≥ 100` → always `true`
    pub fn roll_pct(&mut self, threshold: i32) -> bool {
        if threshold <= 0 {
            return false;
        }
        if threshold >= 100 {
            return true;
        }
        self.0.gen_range(0..100i32) < threshold
    }
}

impl Default for CombatRng {
    fn default() -> Self {
        Self::from_seed(42)
    }
}
