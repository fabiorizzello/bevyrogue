use bevy::prelude::Component;

use super::types::{Attribute, DamageTag, EvoStage, UnitId};

#[derive(Component, Debug, Clone)]
pub struct Unit {
    pub id: UnitId,
    pub name: String,
    pub hp_max: i32,
    pub hp_current: i32,
    pub attribute: Attribute,
    pub resists: Vec<DamageTag>,
    pub evo_stage: EvoStage,
}

impl Unit {
    pub fn is_ko(&self) -> bool {
        self.hp_current <= 0
    }

    pub fn revive(&mut self, pct: i32) {
        if self.is_ko() {
            self.hp_current = (self.hp_max * pct) / 100;
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Commander;

// Used by S06/T02.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ko;

/// Per-team slot index assigned at spawn time (insertion order in apply_composition).
/// Stable across the encounter — never mutated, survives Revive.
/// (Team, SlotIndex) together give global uniqueness.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlotIndex(pub u8);
