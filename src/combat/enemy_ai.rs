//! Pure (no Bevy World access) enemy AI decision function.
//!
//! # Precondition
//! Callers must exclude Commanders and KO'd units from `EnemyTurnContext::targets`.
//! The function assumes every `TargetInfo` in the slice is a valid, living, non-commander
//! ally. Violating this precondition may cause incorrect targeting.

use crate::combat::{kit::UnitSkills, turn_system::ActionIntent, types::UnitId};

/// Snapshot of a single potential target, populated by the caller from Bevy components.
#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub id: UnitId,
    pub toughness_current: i32,
    pub toughness_max: i32,
    pub hp_current: i32,
    pub hp_max: i32,
}

/// All data the decision function needs; carries no Bevy World references.
pub struct EnemyTurnContext<'a> {
    pub attacker_id: UnitId,
    pub attacker_skills: &'a UnitSkills,
    pub attacker_ult_ready: bool,
    pub targets: &'a [TargetInfo],
}

/// Select an `ActionIntent` for an enemy unit.
///
/// Decision priority:
/// 1. **Ultimate** — if `attacker_ult_ready` and at least one target exists.
/// 2. **Skill** — if `attacker_skills.skills` is non-empty (uses the first skill id).
/// 3. **Basic** — fallback.
///
/// In all branches the target is the ally with the lowest
/// `toughness_current / toughness_max` ratio, ties broken by lowest `UnitId.0`.
/// Returns `None` only when `targets` is empty.
pub fn pick_enemy_action(ctx: &EnemyTurnContext<'_>) -> Option<ActionIntent> {
    let target = pick_target(ctx.targets)?;

    let intent = if ctx.attacker_ult_ready {
        ActionIntent::Ultimate {
            attacker: ctx.attacker_id,
            target,
        }
    } else if let Some(skill_id) = ctx.attacker_skills.skills.first() {
        ActionIntent::Skill {
            attacker: ctx.attacker_id,
            skill_id: skill_id.clone(),
            target,
        }
    } else {
        ActionIntent::Basic {
            attacker: ctx.attacker_id,
            target,
        }
    };

    Some(intent)
}

/// Returns the `UnitId` of the target with the lowest toughness ratio,
/// breaking ties by the lowest `UnitId.0`. Returns `None` if `targets` is empty.
fn pick_target(targets: &[TargetInfo]) -> Option<UnitId> {
    targets
        .iter()
        .min_by(|a, b| {
            let ratio_a = toughness_ratio(a);
            let ratio_b = toughness_ratio(b);
            ratio_a
                .partial_cmp(&ratio_b)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.0.cmp(&b.id.0))
        })
        .map(|t| t.id)
}

#[inline]
fn toughness_ratio(t: &TargetInfo) -> f32 {
    if t.toughness_max == 0 {
        return 0.0;
    }
    t.toughness_current as f32 / t.toughness_max as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        kit::UnitSkills,
        types::{SkillId, UnitId},
    };

    fn skills_with(ids: &[&str]) -> UnitSkills {
        UnitSkills {
            basic: SkillId("basic".to_string()),
            skills: ids.iter().map(|s| SkillId(s.to_string())).collect(),
            ultimate: SkillId("ult".to_string()),
            follow_up: None,
        }
    }

    fn target(id: u32, tough_cur: i32, tough_max: i32) -> TargetInfo {
        TargetInfo {
            id: UnitId(id),
            toughness_current: tough_cur,
            toughness_max: tough_max,
            hp_current: 100,
            hp_max: 100,
        }
    }

    #[test]
    fn pick_action_uses_ultimate_when_ready() {
        let skills = skills_with(&["skill_a"]);
        let targets = vec![target(1, 5, 10)];
        let ctx = EnemyTurnContext {
            attacker_id: UnitId(10),
            attacker_skills: &skills,
            attacker_ult_ready: true,
            targets: &targets,
        };
        let intent = pick_enemy_action(&ctx).unwrap();
        assert!(
            matches!(
                intent,
                ActionIntent::Ultimate {
                    attacker: UnitId(10),
                    target: UnitId(1)
                }
            ),
            "expected Ultimate, got {:?}",
            intent
        );
    }

    #[test]
    fn pick_action_uses_skill_when_ult_not_ready() {
        let skills = skills_with(&["skill_b"]);
        let targets = vec![target(2, 8, 10)];
        let ctx = EnemyTurnContext {
            attacker_id: UnitId(10),
            attacker_skills: &skills,
            attacker_ult_ready: false,
            targets: &targets,
        };
        let intent = pick_enemy_action(&ctx).unwrap();
        assert!(
            matches!(
                &intent,
                ActionIntent::Skill {
                    attacker: UnitId(10),
                    skill_id,
                    target: UnitId(2),
                } if skill_id.0 == "skill_b"
            ),
            "expected Skill(skill_b), got {:?}",
            intent
        );
    }

    #[test]
    fn pick_action_falls_back_to_basic_when_no_skills() {
        let skills = skills_with(&[]);
        let targets = vec![target(3, 10, 10)];
        let ctx = EnemyTurnContext {
            attacker_id: UnitId(10),
            attacker_skills: &skills,
            attacker_ult_ready: false,
            targets: &targets,
        };
        let intent = pick_enemy_action(&ctx).unwrap();
        assert!(
            matches!(
                intent,
                ActionIntent::Basic {
                    attacker: UnitId(10),
                    target: UnitId(3)
                }
            ),
            "expected Basic, got {:?}",
            intent
        );
    }

    #[test]
    fn pick_action_targets_lowest_toughness_ratio() {
        // target 1: 8/10 = 0.8, target 2: 3/10 = 0.3 → should pick target 2
        let skills = skills_with(&[]);
        let targets = vec![target(1, 8, 10), target(2, 3, 10)];
        let ctx = EnemyTurnContext {
            attacker_id: UnitId(10),
            attacker_skills: &skills,
            attacker_ult_ready: false,
            targets: &targets,
        };
        let intent = pick_enemy_action(&ctx).unwrap();
        assert!(
            matches!(
                intent,
                ActionIntent::Basic {
                    target: UnitId(2),
                    ..
                }
            ),
            "expected target UnitId(2), got {:?}",
            intent
        );
    }

    #[test]
    fn pick_action_breaks_tie_by_unit_id() {
        // both at 5/10 = 0.5 → tie broken by lowest UnitId
        let skills = skills_with(&[]);
        let targets = vec![target(5, 5, 10), target(2, 5, 10)];
        let ctx = EnemyTurnContext {
            attacker_id: UnitId(10),
            attacker_skills: &skills,
            attacker_ult_ready: false,
            targets: &targets,
        };
        let intent = pick_enemy_action(&ctx).unwrap();
        assert!(
            matches!(
                intent,
                ActionIntent::Basic {
                    target: UnitId(2),
                    ..
                }
            ),
            "expected target UnitId(2) (lowest id), got {:?}",
            intent
        );
    }
}
