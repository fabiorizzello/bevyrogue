use super::*;
use crate::combat::{
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};

mod edge;
mod matrix;

fn make_unit(attr: Attribute, resists: Vec<DamageTag>) -> Unit {
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

fn atk(tag: DamageTag, base: i32, is_break: bool) -> AttackContext {
    AttackContext {
        damage_tag: tag,
        base_damage: base,
        is_break,
    }
}
