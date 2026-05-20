//! Standard unit fixtures for `apply_legacy_ops`-style tests.
//!
//! Eight integration test files used to redefine identical `attacker()` /
//! `defender()` factories; this module is the single source of truth.

#![allow(dead_code)]

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
