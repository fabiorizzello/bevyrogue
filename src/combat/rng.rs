use bevy::prelude::{Commands, Component, Entity, Query, Resource, Single, With, Without, World};
use bevy_prng::WyRand;
use bevy_rand::prelude::{EntropyPlugin, ForkableRng, ForkableSeed, GlobalRng, RngSeed};
use rand_core::{Rng, SeedableRng};

use crate::combat::types::UnitId;
use crate::combat::unit::Unit;

pub const DEFAULT_COMBAT_RNG_SEED: u64 = 0xDEAD_BEEF;

/// Bevy-native combat entropy algorithm.
pub type CombatEntropy = WyRand;

/// Seed component for per-entity combat RNG streams.
pub type CombatRngSeed = RngSeed<CombatEntropy>;

/// Marker for entities whose RNG stream was forked from the global combat source.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnitRng;

/// Centralised combat RNG resource (R019).
///
/// This wraps the same `bevy_prng::WyRand` entropy source that `bevy_rand`
/// registers for global/per-entity ECS RNGs. All combat randomness stays seeded,
/// replayable, and forkable without using thread-local entropy or direct `rand 0.8`
/// game-path APIs.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct CombatRng {
    entropy: CombatEntropy,
}

impl CombatRng {
    /// Seed from a single `u64`, expanded into the PRNG seed byte buffer.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            entropy: CombatEntropy::from_seed(combat_seed_from_u64(seed)),
        }
    }

    /// Fork a standalone RNG stream from this resource.
    pub fn fork_rng(&mut self) -> CombatEntropy {
        self.entropy.fork_rng()
    }

    /// Fork a seed component that can be attached to an entity. `bevy_rand`
    /// materializes the corresponding `CombatEntropy` component from this seed.
    pub fn fork_seed(&mut self) -> CombatRngSeed {
        self.entropy.fork_seed()
    }

    /// Returns `true` if a uniform draw in `[0, 100)` is strictly less than
    /// `threshold`. Delegates to [`roll_pct_entropy`] so the global resource and
    /// per-entity streams share identical roll math (same draws -> same outcome).
    pub fn roll_pct(&mut self, threshold: i32) -> bool {
        roll_pct_entropy(&mut self.entropy, threshold)
    }
}

/// Uniform `[0, upper_exclusive)` draw with rejection sampling (no modulo bias).
fn next_below(entropy: &mut CombatEntropy, upper_exclusive: u32) -> u32 {
    debug_assert!(upper_exclusive > 0);
    let rejection_zone = u32::MAX - (u32::MAX % upper_exclusive);
    loop {
        let value = entropy.next_u32();
        if value < rejection_zone {
            return value % upper_exclusive;
        }
    }
}

/// Percent roll against any combat entropy stream. Boundary cases are clamped
/// and consume **no** entropy:
///   - `threshold <= 0` -> always `false`
///   - `threshold >= 100` -> always `true`
///
/// This is the single canonical roll primitive: [`CombatRng::roll_pct`] (the
/// global resource) and per-entity `CombatEntropy` rolls both go through it, so
/// a forked stream and the resource produce the same outcome for the same draw.
pub fn roll_pct_entropy(entropy: &mut CombatEntropy, threshold: i32) -> bool {
    if threshold <= 0 {
        return false;
    }
    if threshold >= 100 {
        return true;
    }
    next_below(entropy, 100) < threshold as u32
}

/// Roll a percentage check against `unit`'s own per-entity `CombatEntropy`
/// stream from inside an exclusive (`&mut World`) system.
///
/// The unit's forked stream (seeded from the global source by
/// [`seed_unit_rngs`]) is the canonical source: the entity that owns the random
/// decision rolls from its own replayable stream. When that unit has no forked
/// stream yet — only minimal test fixtures that never run `seed_unit_rngs` — it
/// falls back to the **persistent** seeded [`CombatRng`] resource (inserted on
/// first use so successive rolls keep advancing). There is no per-roll
/// throwaway RNG.
pub fn roll_pct_for_unit_in_world(world: &mut World, unit: UnitId, threshold: i32) -> bool {
    let mut q = world.query::<(&Unit, &mut CombatEntropy)>();
    for (u, mut entropy) in q.iter_mut(world) {
        if u.id == unit {
            return roll_pct_entropy(&mut entropy, threshold);
        }
    }
    world
        .get_resource_or_insert_with(CombatRng::default)
        .roll_pct(threshold)
}

/// Roll status accuracy for `attacker` — the timeline-backed apply path.
///
/// Thin alias over [`roll_pct_for_unit_in_world`] kept for call-site clarity:
/// accuracy is rolled from the attacker's own per-entity stream.
pub fn roll_accuracy_in_world(world: &mut World, attacker: UnitId, threshold: i32) -> bool {
    roll_pct_for_unit_in_world(world, attacker, threshold)
}

impl Default for CombatRng {
    fn default() -> Self {
        Self::from_seed(42)
    }
}

pub fn combat_seed_from_u64(seed: u64) -> <CombatEntropy as SeedableRng>::Seed {
    let mut out = <CombatEntropy as SeedableRng>::Seed::default();
    let bytes = seed.to_le_bytes();
    let seed_bytes = out.as_mut();
    let len = bytes.len().min(seed_bytes.len());
    seed_bytes[..len].copy_from_slice(&bytes[..len]);
    out
}

pub fn combat_entropy_plugin_from_seed(seed: u64) -> EntropyPlugin<CombatEntropy> {
    EntropyPlugin::<CombatEntropy>::with_seed(combat_seed_from_u64(seed))
}

/// Attach deterministic per-entity RNG streams to units spawned without one.
///
/// The stream is forked from the global `bevy_rand` source, so spawn order is the
/// only input that determines entity RNG seeds for a fixed combat seed.
pub fn seed_unit_rngs(
    mut commands: Commands,
    mut global: Single<&mut CombatEntropy, With<GlobalRng>>,
    units: Query<Entity, (With<Unit>, Without<CombatEntropy>, Without<CombatRngSeed>)>,
) {
    for entity in &units {
        commands
            .entity(entity)
            .insert((UnitRng, global.fork_seed()));
    }
}
