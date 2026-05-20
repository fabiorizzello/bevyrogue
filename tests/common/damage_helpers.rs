//! Shared fixtures for `combat::mechanics::damage` integration tests.
//! Relocated from `src/combat/mechanics/damage/tests/mod.rs` (R003).

#![allow(dead_code)]

use bevyrogue::combat::mechanics::damage::AttackContext;
use bevyrogue::combat::types::{Attribute, DamageTag, EvoStage, UnitId};
use bevyrogue::combat::unit::Unit;

pub fn make_unit(attr: Attribute, resists: Vec<DamageTag>) -> Unit {
    Unit {
        id: UnitId(0),
        name: String::new(),
        hp_max: 100,
        hp_current: 100,
        attribute: attr,
        resists,
        evo_stage: EvoStage::Adult,
    }
}

pub fn atk(tag: DamageTag, base: i32, is_break: bool) -> AttackContext {
    AttackContext {
        damage_tag: tag,
        base_damage: base,
        is_break,
    }
}
