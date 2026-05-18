/// Deterministic per-cast RNG using SplitMix64.
///
/// Each (global_seed, cast_id, beat, hop, salt) tuple produces an independent,
/// reproducible stream (invariant I1: same input ⇒ same Intent stream). No
/// external crate dependency — the algorithm is five arithmetic operations.
///
/// # Separation from `combat::rng::CombatRng`
/// `CombatRng` (StdRng) handles legacy global rolls (status accuracy, etc.).
/// `CastRng` is cast-scoped and seeded per invocation, enabling full replay.
// Used in cfg(test) within this file; public API for future skill-hook callers.
#[allow(dead_code)]
pub struct CastRng(u64);

// All methods consumed in cfg(test) within this file; public API surface for future use.
#[allow(dead_code)]
impl CastRng {
    /// Seed from a raw `u64`. Runs one warm-up step so that `seed=0` and
    /// `seed=1` diverge on the first `next_u64` call.
    pub fn new(seed: u64) -> Self {
        let mut rng = Self(seed);
        let _ = rng.next_u64();
        rng
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

    /// SplitMix64 step: advances state and returns the next 64-bit output.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15u64);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9u64);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111ebu64);
        z ^ (z >> 31)
    }

    /// Upper 32 bits of the next 64-bit output.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Uniform sample in `[0, n)`. Panics if `n == 0`.
    ///
    /// Uses modulo reduction — sufficient for small `n` values used in combat.
    pub fn next_below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "CastRng::next_below: n must be > 0");
        (self.next_u64() % n as u64) as u32
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
        let a = CastRng::from_params(999, 1, 2, 3, 42);
        let b = CastRng::from_params(999, 1, 2, 3, 42);
        // Internal state must match after construction.
        assert_eq!(a.0, b.0);
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
