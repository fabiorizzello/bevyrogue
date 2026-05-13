use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use super::status_effect::StatusEffectKind;
use super::types::{Attribute, DamageTag, SkillId};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FollowUpTrigger {
    OnEnemyBreak,
    OnAllyLowHp,
    OnEnemyKill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FollowUpConfig {
    pub trigger: FollowUpTrigger,
    pub action: SkillId,
}

// Used by S06/T02.
#[allow(dead_code)]
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct UnitSkills {
    pub basic: SkillId,
    pub skills: Vec<SkillId>,
    pub ultimate: SkillId,
    pub follow_up: Option<FollowUpConfig>,
}

/// Trigger condition for Form Identity — once-per-round conditional bonuses.
/// All four variants declared upfront; only OnFirstHitVsTagThisRound is wired in T02.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormIdentityTrigger {
    /// Fires on the first hit with the given damage tag dealt by this unit this round.
    /// Tag specificity checking deferred to T03 (requires damage_tag in OnDamageDealt).
    OnFirstHitVsTagThisRound(DamageTag),
    /// Fires when this unit applies the given status effect.
    OnStatusApplied(StatusEffectKind),
    /// Fires on the first skill cast with the given tag this round.
    OnFirstSkillCastWithTag(DamageTag),
    /// Fires when this unit attacks an enemy with the given attribute.
    OnAttackVsAttribute(Attribute),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormIdentityConfig {
    pub trigger: FormIdentityTrigger,
    pub action: SkillId,
}

/// ECS component carrying Form Identity configuration.
/// Only spawned for units that have `form_identity` set in their UnitDef.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct FormIdentityKit {
    pub config: FormIdentityConfig,
}
