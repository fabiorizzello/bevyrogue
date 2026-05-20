//! Shared magic-number constants and canonical `UnitId`s used across
//! integration tests. Lifted here so future renames (e.g. roster expansion)
//! are single-line changes.

#![allow(dead_code)]

use bevyrogue::combat::types::UnitId;

/// Default HP pool used by most fixtures.
pub const BASIC_HP: i32 = 100;
/// Larger pool for damage-scaling tests that want headroom.
pub const LARGE_HP: i32 = 1_000;
/// Default SP pool starting value and max in `tests/common::build_app`.
pub const BASIC_SP: i32 = 5;
/// Conventional "wounded" threshold (≤25% hp_pct) used by passive triggers.
pub const LOW_HP_THRESHOLD: i32 = 25;

// --- Canonical party `UnitId`s ---------------------------------------------
// These mirror the roster order in `assets/data/party.ron`. Tests that hard-
// code numeric `UnitId(n)` should import these instead.

pub const AGUMON_ID: UnitId = UnitId(1);
pub const GABUMON_ID: UnitId = UnitId(2);
pub const DORUMON_ID: UnitId = UnitId(3);
pub const TENTOMON_ID: UnitId = UnitId(4);
pub const PATAMON_ID: UnitId = UnitId(9);
pub const RENAMON_ID: UnitId = UnitId(7);

/// Generic ally dummy used when the test just needs a body on Team::Ally.
pub const ALLY_DUMMY_ID: UnitId = UnitId(42);
/// Generic enemy dummy used when the test just needs a body on Team::Enemy.
pub const ENEMY_DUMMY_ID: UnitId = UnitId(99);

// --- Conventional seeds ----------------------------------------------------
// The test suite uses 42 as the "neutral" seed. Tests probing RNG drift use
// `DRIFT_SEED` to assert a different deterministic path.

pub const NEUTRAL_SEED: u64 = 42;
pub const DRIFT_SEED: u64 = 1337;
