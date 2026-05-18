use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::skill_timeline::SkillTimeline;
use crate::combat::status_effect::StatusEffectKind;
use crate::combat::types::{DamageTag, SkillId};

/// How the next bounce hop target is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BounceSelector {
    /// Select the alive enemy with the lowest HP percentage.
    LowestHpPctAlive,
    /// Select the next alive enemy in slot order (wrapping).
    NextSlotAlive,
    /// Select the alive enemy in the adjacent slot with the lowest HP.
    AdjLowest,
}

/// Whether the bounce chain is allowed to revisit already-hit targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatPolicy {
    /// Each target can only be hit once per cast.
    NoRepeat,
    /// Targets may be re-selected on subsequent hops.
    AllowRepeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetShape {
    Single,
    /// Primary target + adjacent slot_index ±1 on the same team, alive, slot_index asc.
    Blast,
    Row,
    AllEnemies,
    SelfOnly,
    /// All alive units on the caster's own team (ally side), slot_index ascending.
    AllAllies,
    /// Chaining bounce: hits up to `hops` targets in sequence, re-resolving the selector
    /// each hop. Chain stops early if no valid target is found.
    Bounce {
        hops: u8,
        selector: BounceSelector,
        repeat: RepeatPolicy,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetSide {
    Ally,
    Enemy,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetLife {
    Alive,
    Ko,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelfTargetRule {
    Forbid,
    Allow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TargetHpRule {
    #[default]
    Any,
    Damaged,
}

// S03 declares side/life/self targeting metadata here; later slices make it queryable and enforce it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillTargeting {
    pub shape: TargetShape,
    pub side: TargetSide,
    pub life: TargetLife,
    pub self_rule: SelfTargetRule,
    #[serde(default)]
    pub target_hp_rule: TargetHpRule,
}

impl Default for SkillTargeting {
    fn default() -> Self {
        Self {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LegalityReasonCode {
    UnimplementedTargetShape,
    UnimplementedEffect,
    WrongSide,
    TargetKo,
    TargetNotKo,
    TargetFullHp,
    TargetNotDamaged,
    TargetIsSelf,
    TargetIsCommander,
    NoValidTargets,
    ToughnessEnemyOnly,
    NotActiveUnit,
    WrongPhase,
    AttackerKo,
    AttackerStunned,
    MissingSkill,
    SpShortfall,
    UltimateNotReady,
    TargetNotFound,
    TamerGaugeDeferred,
    TamerCommandDeferred,
    ChargedTelegraphDeferred,
    EnemyTraitDeferred,
    EnergyCapReached,
    /// A skill carries two effect kinds that are mutually exclusive in v0 (e.g. Heal + Cleanse).
    MixedEffectKinds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkillImplementation {
    #[default]
    Implemented,
    Deferred {
        reason: LegalityReasonCode,
    },
    Hidden {
        reason: LegalityReasonCode,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CustomSignalPayload {
    Empty,
    Amount { amount: i32 },
}

impl Default for CustomSignalPayload {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillCustomSignal {
    pub owner: String,
    pub signal: String,
    #[serde(default)]
    pub payload: CustomSignalPayload,
}

impl SkillCustomSignal {
    pub fn blueprint(
        owner: impl Into<String>,
        signal: impl Into<String>,
        payload: CustomSignalPayload,
    ) -> Self {
        Self {
            owner: owner.into(),
            signal: signal.into(),
            payload,
        }
    }

    pub fn owner(&self) -> &str {
        self.owner.as_str()
    }

    pub fn signal(&self) -> &str {
        self.signal.as_str()
    }

    pub fn payload(&self) -> CustomSignalPayload {
        self.payload
    }
}

/// Per-hop damage scaling for Bounce chains.
///
/// - `Constant`: every hop deals `base_damage` (default).
/// - `Falloff { pct }`: each subsequent hop deals `pct/100` of `base_damage` less than the previous
///   (i.e. hop N deals `base_damage * (pct/100)^N`). `pct` must be <= 100.
/// - `PerHop(Vec<i32>)`: explicit override per hop; vec length must equal `hops`.
///   Overrides `base_damage` for each index; `base_damage` is ignored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DamageCurve {
    #[default]
    Constant,
    Falloff {
        /// Percentage retained per hop (1–100). E.g. 80 means each hop deals 80% of the previous.
        pct: u16,
    },
    PerHop(Vec<i32>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LegacyEffect {
    Damage {
        amount: i32,
        target: TargetShape,
        #[serde(default)]
        per_hop: DamageCurve,
    },
    ToughnessHit(i32),
    GainSP(i32),
    UltGain(i32),
    Stun,
    Revive(i32),
    GrantFreeSkill {
        count: usize,
    },
    ApplyStatus {
        kind: StatusEffectKind,
        duration: u32,
    },
    AdvanceTurn(u32),
    DelayTurn(u32),
    /// Grant the attacker N energy (once-per-round gated by RoundFlags.form_identity_used).
    GrantEnergy(i32),
    /// Advance the attacker's own AV by N percent (self-tempo boost).
    SelfAdvance(i32),
    /// Restore HP to one or more allies. `amount_pct_max_hp` is a percentage of the target's
    /// hp_max (1–100). `target` must be an ally-side shape (Single, SelfOnly, AllAllies).
    /// Capped at hp_max; no-ops silently on KO targets.
    Heal {
        amount_pct_max_hp: u32,
        target: TargetShape,
    },
    /// Remove up to `count` non-immune debuffs from an ally's StatusBag (None = remove all).
    /// `target` must be an ally-side shape (Single, SelfOnly, AllAllies).
    /// Cannot coexist with Effect::Heal in the same skill (deferred to M021).
    Cleanse {
        count: Option<u8>,
        target: TargetShape,
    },
}

pub use LegacyEffect as Effect;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SkillDef {
    pub id: SkillId,
    pub name: String,
    pub damage_tag: DamageTag,
    pub sp_cost: i32,
    pub targeting: SkillTargeting,
    pub implementation: SkillImplementation,
    pub legacy_ops: Vec<LegacyEffect>,
    #[serde(default)]
    pub custom_signals: Vec<SkillCustomSignal>,
    /// Optional sequence of animation steps for visual polish.
    pub animation_sequence: Option<Vec<String>>,
    /// Optional QTE mechanic description.
    pub qte: Option<String>,
    /// Optional compiled timeline schema for the kernel timeline path.
    #[serde(default)]
    pub timeline: Option<SkillTimeline>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillBookValidationCategory {
    // kept for: structural-error category (vocabulary anchor; only Semantic constructed today)
    Structural,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillBookValidationError {
    pub skill_id: SkillId,
    pub category: SkillBookValidationCategory,
    pub reason: LegalityReasonCode,
    pub detail: String,
}

impl fmt::Display for SkillBookValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "skill_id={} category={:?} reason={:?} detail={}",
            self.skill_id.0, self.category, self.reason, self.detail
        )
    }
}

impl std::error::Error for SkillBookValidationError {}

pub fn validate_skill_book(book: &SkillBook) -> Result<(), SkillBookValidationError> {
    for skill in &book.0 {
        validate_skill_def(skill)?;
    }
    Ok(())
}

fn validation_error(
    skill: &SkillDef,
    category: SkillBookValidationCategory,
    reason: LegalityReasonCode,
    detail: impl Into<String>,
) -> SkillBookValidationError {
    SkillBookValidationError {
        skill_id: skill.id.clone(),
        category,
        reason,
        detail: detail.into(),
    }
}

fn skill_has_effect(skill: &SkillDef, predicate: impl Fn(&Effect) -> bool) -> bool {
    skill.legacy_ops.iter().any(predicate)
}

const CANON_STATUS_IDS: &[&str] = &["heated", "chilled", "paralyzed", "slowed", "blessed"];

fn validate_skill_def(skill: &SkillDef) -> Result<(), SkillBookValidationError> {
    use crate::combat::status_effect::StatusEffectKind;

    for effect in &skill.legacy_ops {
        if let Effect::ApplyStatus { kind, .. } = effect {
            if matches!(kind, StatusEffectKind::Burn | StatusEffectKind::Shock) {
                return Err(validation_error(
                    skill,
                    SkillBookValidationCategory::Semantic,
                    LegalityReasonCode::UnimplementedEffect,
                    format!(
                        "ApplyStatus uses reserved status kind {:?}; valid ids are: {}",
                        kind,
                        CANON_STATUS_IDS.join(", ")
                    ),
                ));
            }
        }
    }

    let has_damage = skill_has_effect(skill, |effect| matches!(effect, Effect::Damage { .. }));
    let has_revive = skill_has_effect(skill, |effect| matches!(effect, Effect::Revive(_)));

    // Reject Bounce with hops == 0 (always, regardless of implementation status).
    if let TargetShape::Bounce { hops, .. } = skill.targeting.shape {
        if hops == 0 {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::UnimplementedTargetShape,
                "Bounce hops must be >= 1; found hops=0",
            ));
        }
    }

    fn shape_is_executable(shape: TargetShape) -> bool {
        matches!(
            shape,
            TargetShape::Single
                | TargetShape::Blast
                | TargetShape::AllEnemies
                | TargetShape::SelfOnly
                | TargetShape::AllAllies
                | TargetShape::Bounce { .. }
        )
    }

    if matches!(skill.implementation, SkillImplementation::Implemented)
        && !shape_is_executable(skill.targeting.shape)
    {
        return Err(validation_error(
            skill,
            SkillBookValidationCategory::Semantic,
            LegalityReasonCode::UnimplementedTargetShape,
            format!(
                "implemented skills support Single, Blast, AllEnemies, SelfOnly, AllAllies, or Bounce{{hops>=1}}; found {:?}",
                skill.targeting.shape
            ),
        ));
    }

    if has_damage {
        for (target, per_hop) in skill.legacy_ops.iter().filter_map(|effect| match effect {
            Effect::Damage {
                target, per_hop, ..
            } => Some((*target, per_hop)),
            _ => None,
        }) {
            if target != skill.targeting.shape {
                return Err(validation_error(
                    skill,
                    SkillBookValidationCategory::Semantic,
                    LegalityReasonCode::UnimplementedTargetShape,
                    format!(
                        "damage effect target {:?} contradicts targeting.shape {:?}",
                        target, skill.targeting.shape
                    ),
                ));
            }

            // Validate DamageCurve constraints for Bounce shapes.
            if let TargetShape::Bounce { hops, .. } = skill.targeting.shape {
                match per_hop {
                    DamageCurve::Constant => {}
                    DamageCurve::Falloff { pct } => {
                        if *pct > 100 {
                            return Err(validation_error(
                                skill,
                                SkillBookValidationCategory::Semantic,
                                LegalityReasonCode::UnimplementedEffect,
                                format!(
                                    "DamageCurve::Falloff pct must be <= 100; found pct={}",
                                    pct
                                ),
                            ));
                        }
                    }
                    DamageCurve::PerHop(v) => {
                        if v.len() != hops as usize {
                            return Err(validation_error(
                                skill,
                                SkillBookValidationCategory::Semantic,
                                LegalityReasonCode::UnimplementedEffect,
                                format!(
                                    "DamageCurve::PerHop length {} must equal hops {}",
                                    v.len(),
                                    hops
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    if has_revive && !has_damage {
        if skill.targeting.side != TargetSide::Ally {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::WrongSide,
                format!(
                    "revive skills must target allies, found {:?}",
                    skill.targeting.side
                ),
            ));
        }

        if skill.targeting.life != TargetLife::Ko {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::TargetNotKo,
                format!(
                    "revive skills must target KO units, found {:?}",
                    skill.targeting.life
                ),
            ));
        }

        if skill.targeting.shape != TargetShape::Single {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::UnimplementedTargetShape,
                format!(
                    "revive skills currently support only TargetShape::Single, found {:?}",
                    skill.targeting.shape
                ),
            ));
        }
    }

    if has_damage && has_revive {
        match &skill.implementation {
            SkillImplementation::Implemented => {
                return Err(validation_error(
                    skill,
                    SkillBookValidationCategory::Semantic,
                    LegalityReasonCode::UnimplementedEffect,
                    "mixed damage+revive semantics are unresolved",
                ));
            }
            SkillImplementation::Deferred { reason } | SkillImplementation::Hidden { reason } => {
                if *reason != LegalityReasonCode::UnimplementedEffect {
                    return Err(validation_error(
                        skill,
                        SkillBookValidationCategory::Semantic,
                        LegalityReasonCode::UnimplementedEffect,
                        format!(
                            "mixed damage+revive semantics require Deferred/Hidden reason UnimplementedEffect, found {:?}",
                            reason
                        ),
                    ));
                }
            }
        }
    }

    if let SkillImplementation::Deferred { reason } | SkillImplementation::Hidden { reason } =
        &skill.implementation
    {
        if *reason == LegalityReasonCode::UnimplementedTargetShape
            && skill.targeting.shape == TargetShape::Single
            && !has_revive
            && !has_damage
        {
            return Err(validation_error(
                skill,
                SkillBookValidationCategory::Semantic,
                LegalityReasonCode::UnimplementedTargetShape,
                "target-shape deferrals must not claim single-target execution",
            ));
        }
    }

    for effect in &skill.legacy_ops {
        if let Effect::Heal { target, .. } = effect {
            match target {
                TargetShape::Bounce { .. } | TargetShape::AllEnemies | TargetShape::Blast => {
                    return Err(validation_error(
                        skill,
                        SkillBookValidationCategory::Semantic,
                        LegalityReasonCode::WrongSide,
                        format!(
                            "Heal effect may not target enemy-side shapes; found {:?}",
                            target
                        ),
                    ));
                }
                _ => {}
            }
        }
    }

    for effect in &skill.legacy_ops {
        if let Effect::Cleanse { target, .. } = effect {
            match target {
                TargetShape::Bounce { .. } | TargetShape::AllEnemies | TargetShape::Blast => {
                    return Err(validation_error(
                        skill,
                        SkillBookValidationCategory::Semantic,
                        LegalityReasonCode::WrongSide,
                        format!(
                            "Cleanse effect may not target enemy-side shapes; found {:?}",
                            target
                        ),
                    ));
                }
                _ => {}
            }
        }
    }

    let has_heal = skill
        .legacy_ops
        .iter()
        .any(|e| matches!(e, Effect::Heal { .. }));
    let has_cleanse = skill
        .legacy_ops
        .iter()
        .any(|e| matches!(e, Effect::Cleanse { .. }));
    if has_heal && has_cleanse {
        return Err(validation_error(
            skill,
            SkillBookValidationCategory::Semantic,
            LegalityReasonCode::MixedEffectKinds,
            "Heal and Cleanse may not coexist in the same skill (deferred to M021)".to_string(),
        ));
    }

    Ok(())
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct SkillBook(pub Vec<SkillDef>);

#[cfg(test)]
mod tests;
