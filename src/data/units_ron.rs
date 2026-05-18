use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// `EnemyCounterplayKind` is re-consumed by integration tests through
// `bevyrogue::data::units_ron::EnemyCounterplayKind`; the lib alone treats it as unused.
pub use crate::combat::counterplay::{
    EnemyCounterplayKind, ImplementationStatus as EnemyCounterplayStatus,
};
use crate::combat::kit::{FollowUpConfig, FormIdentityConfig};
use crate::combat::team::Team;
use crate::combat::toughness::ToughnessCategory;
use crate::combat::types::{Attribute, DamageTag, EvoLineId, EvoStage, SkillId, UnitId};
use crate::combat::ultimate::UltAccumulationTrigger;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct BlueprintRosterPayload(pub BTreeMap<String, String>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct BlueprintRoster(pub BTreeMap<String, BlueprintRosterPayload>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitDef {
    pub id: UnitId,
    pub name: String,
    pub role_tags: Vec<String>,
    pub signature_traits: Vec<String>,
    pub hp_max: i32,
    pub attribute: Attribute,
    // routing field: Ally for player units, Enemy for opponents
    pub team: Team,
    pub basic_damage_tag: DamageTag,
    pub basic_skill: SkillId,
    pub skill_ids: Vec<SkillId>,
    pub ultimate_skill: SkillId,
    pub follow_up: Option<FollowUpConfig>,
    #[serde(default)]
    pub enemy_traits: Vec<crate::combat::counterplay::EnemyTraitDeclaration>,
    #[serde(default)]
    pub charged_attack: Option<crate::combat::counterplay::ChargedAttackDeclaration>,
    /// Once-per-round conditional bonus; absent in RON for units without form identity.
    #[serde(default)]
    pub form_identity: Option<FormIdentityConfig>,
    /// Owner-keyed blueprint roster metadata used by contract tests and kernel-adjacent hooks.
    #[serde(default)]
    pub blueprint_metadata: BlueprintRoster,
    pub resists: Vec<DamageTag>,
    pub toughness_max: i32,
    pub weaknesses: Vec<DamageTag>,
    pub ultimate_trigger: i32,
    pub ultimate_cap: i32,
    pub ultimate_accumulation_trigger: UltAccumulationTrigger,
    pub ultimate_charge_per_event: i32,
    pub speed: i32,
    pub evo_stage: EvoStage,
    pub evo_line: EvoLineId,
    pub evolves_to: Vec<UnitId>,
    /// Boss units with this flag get a `TempoResistance` component on spawn.
    #[serde(default)]
    pub tempo_resistant: bool,
    /// Toughness defensive archetype; defaults to Standard for backward compatibility.
    #[serde(default)]
    pub toughness_category: ToughnessCategory,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct UnitRoster(pub Vec<UnitDef>);

#[cfg(test)]
mod tests;
