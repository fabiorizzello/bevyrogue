use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::combat::types::SkillId;
pub use crate::data::skills_ron::LegalityReasonCode;
use crate::data::units_ron::UnitDef;
// Consumed by integration tests via `bevyrogue::combat::counterplay::EnemyCounterplayStatus`;
// the lib target alone does not see the usage, hence the explicit allow.
pub use ImplementationStatus as EnemyCounterplayStatus;

/// Typed enemy counterplay declarations exposed through unit data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyCounterplayKind {
    TypeTrap,
    ReactiveArmor,
    BreakSeal,
    TempoAnchor,
}

/// Queryable implementation state shared by enemy trait and charged-telegraph declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplementationStatus {
    Implemented,
    Deferred { reason: LegalityReasonCode },
    Hidden { reason: LegalityReasonCode },
}

impl Default for ImplementationStatus {
    fn default() -> Self {
        Self::Implemented
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyTraitDeclaration {
    pub kind: EnemyCounterplayKind,
    #[serde(default)]
    pub status: ImplementationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargedAttackDeclaration {
    pub skill_id: SkillId,
    pub lead_turns: u8,
    #[serde(default)]
    pub status: ImplementationStatus,
}

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
