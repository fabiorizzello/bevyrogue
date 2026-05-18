use thiserror::Error;

use crate::combat::types::SkillId;

use super::types::{
    DamageCurve, Effect, LegalityReasonCode, SkillBook, SkillDef, SkillImplementation, TargetLife,
    TargetShape, TargetSide,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillBookValidationCategory {
    // kept for: structural-error category (vocabulary anchor; only Semantic constructed today)
    Structural,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("skill_id={} category={:?} reason={:?} detail={detail}", self.skill_id.0, self.category, self.reason)]
pub struct SkillBookValidationError {
    pub skill_id: SkillId,
    pub category: SkillBookValidationCategory,
    pub reason: LegalityReasonCode,
    pub detail: String,
}

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
