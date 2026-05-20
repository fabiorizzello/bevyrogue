//! Standard unit fixtures for `apply_legacy_ops`-style tests.
//!
//! Eight integration test files used to redefine identical `attacker()` /
//! `defender()` factories; this module is the single source of truth.

#![allow(dead_code)]

use bevy::prelude::*;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::{Attribute, EvoStage, UnitId};
use bevyrogue::combat::unit::Unit;

pub const ATTACKER_ID: UnitId = UnitId(1);
pub const DEFENDER_ID: UnitId = UnitId(2);

pub fn unit(id: UnitId, name: &str, attribute: Attribute) -> Unit {
    Unit {
        id,
        name: name.into(),
        hp_max: 1_000,
        hp_current: 1_000,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

pub fn attacker() -> Unit {
    unit(ATTACKER_ID, "attacker", Attribute::Data)
}

pub fn defender() -> Unit {
    unit(DEFENDER_ID, "defender", Attribute::Data)
}

/// Lower-level factory: build a `Unit` with explicit `hp_max == hp_current`.
/// Used by the broader `tests/` set where the `Adult / Attribute::Data`
/// default works (timeline + intent tests).
pub fn make_unit(id: UnitId, name: &str, hp: i32) -> Unit {
    Unit {
        id,
        name: name.into(),
        hp_max: hp,
        hp_current: hp,
        attribute: Attribute::Data,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

/// Spawn a unit entity onto the test world with `(Unit, Team)`. Returns the
/// `Entity`. Mirrors the most common signature found across 9 test files.
pub fn spawn_unit(app: &mut App, id: UnitId, team: Team, hp: i32) -> Entity {
    let name = format!("unit_{}", id.0);
    app.world_mut().spawn((make_unit(id, &name, hp), team)).id()
}

/// Same as [`spawn_unit`] but lets the caller name the unit (used by tests
/// that assert on `Unit.name`).
pub fn spawn_named_unit(app: &mut App, id: UnitId, name: &str, team: Team, hp: i32) -> Entity {
    app.world_mut().spawn((make_unit(id, name, hp), team)).id()
}
