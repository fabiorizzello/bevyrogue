use bevy::prelude::Component;

use crate::combat::counterplay::{ChargedAttackDeclaration, EnemyTraitDeclaration};
use crate::data::units_ron::UnitDef;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub struct EnemyCounterplayKit {
    pub enemy_traits: Vec<EnemyTraitDeclaration>,
    pub charged_attack: Option<ChargedAttackDeclaration>,
}

impl EnemyCounterplayKit {
    pub fn from_def(def: &UnitDef) -> Option<Self> {
        if def.enemy_traits.is_empty() && def.charged_attack.is_none() {
            None
        } else {
            Some(Self {
                enemy_traits: def.enemy_traits.clone(),
                charged_attack: def.charged_attack.clone(),
            })
        }
    }
}
