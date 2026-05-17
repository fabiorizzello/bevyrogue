//! Pure (no Bevy World access) enemy AI decision function.
//!
//! # Precondition
//! Callers must exclude Commanders and KO'd units from `EnemyTurnContext::targets`.
//! The function assumes every `TargetInfo` in the slice is a valid, living, non-commander
//! ally. Violating this precondition may cause incorrect targeting.

use std::cmp::Ordering;

use crate::combat::{
    kit::UnitSkills,
    preview::PreviewDamageSummary,
    turn_system::ActionIntent,
    types::{SkillId, UnitId},
};

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

#[derive(Debug, Clone)]
struct PreviewScoredAction {
    intent: ActionIntent,
    damage: i32,
    action_rank: u8,
    target_id: UnitId,
    skill_rank: usize,
}

fn action_rank(intent: &ActionIntent) -> u8 {
    match intent {
        ActionIntent::Basic { .. } => 0,
        ActionIntent::Skill { .. } => 1,
        ActionIntent::Ultimate { .. } => 2,
    }
}

fn compare_scored_action(a: &PreviewScoredAction, b: &PreviewScoredAction) -> Ordering {
    a.damage
        .cmp(&b.damage)
        .then_with(|| a.action_rank.cmp(&b.action_rank))
        .then_with(|| b.target_id.0.cmp(&a.target_id.0))
        .then_with(|| b.skill_rank.cmp(&a.skill_rank))
}

fn build_basic_intent(attacker_id: UnitId, target: UnitId) -> ActionIntent {
    ActionIntent::Basic {
        attacker: attacker_id,
        target,
    }
}

fn build_skill_intent(attacker_id: UnitId, skill_id: SkillId, target: UnitId) -> ActionIntent {
    ActionIntent::Skill {
        attacker: attacker_id,
        skill_id,
        target,
    }
}

fn build_ultimate_intent(attacker_id: UnitId, target: UnitId) -> ActionIntent {
    ActionIntent::Ultimate {
        attacker: attacker_id,
        target,
    }
}

/// Select an `ActionIntent` for an enemy unit.
///
/// If preview data is available, the best action/target pair is the one with the
/// highest summed preview damage. Ties are deterministic: ultimate beats skill,
/// skill beats basic, lower `UnitId` wins on the target axis, and earlier skills
/// in the list win when all else is equal.
///
/// If no preview data can be produced for any candidate, the function falls back
/// to the legacy deterministic routing: ultimate if ready, otherwise first skill,
/// otherwise basic, all against the lowest-toughness target.
pub fn pick_enemy_action(ctx: &EnemyTurnContext<'_>) -> Option<ActionIntent> {
    pick_enemy_action_with_preview(ctx, |_skill_id, _target| None)
}

/// Preview-aware variant used by the runtime bridge and preview-driven tests.
pub fn pick_enemy_action_with_preview<F>(
    ctx: &EnemyTurnContext<'_>,
    mut preview_for: F,
) -> Option<ActionIntent>
where
    F: FnMut(&SkillId, UnitId) -> Option<PreviewDamageSummary>,
{
    let fallback_target = pick_target(ctx.targets)?;
    let fallback_intent = if ctx.attacker_ult_ready {
        build_ultimate_intent(ctx.attacker_id, fallback_target)
    } else if let Some(skill_id) = ctx.attacker_skills.skills.first() {
        build_skill_intent(ctx.attacker_id, skill_id.clone(), fallback_target)
    } else {
        build_basic_intent(ctx.attacker_id, fallback_target)
    };

    let mut scored_actions = Vec::new();
    let mut targets: Vec<UnitId> = ctx.targets.iter().map(|target| target.id).collect();
    targets.sort_by(|a, b| a.0.cmp(&b.0));

    for target in targets.iter().copied() {
        if let Some(summary) = preview_for(&ctx.attacker_skills.basic, target) {
            scored_actions.push(PreviewScoredAction {
                intent: build_basic_intent(ctx.attacker_id, target),
                damage: summary.total_damage,
                action_rank: 0,
                target_id: target,
                skill_rank: 0,
            });
        }
    }

    for (skill_rank, skill_id) in ctx.attacker_skills.skills.iter().enumerate() {
        for target in targets.iter().copied() {
            if let Some(summary) = preview_for(skill_id, target) {
                scored_actions.push(PreviewScoredAction {
                    intent: build_skill_intent(ctx.attacker_id, skill_id.clone(), target),
                    damage: summary.total_damage,
                    action_rank: 1,
                    target_id: target,
                    skill_rank,
                });
            }
        }
    }

    if ctx.attacker_ult_ready {
        for target in targets.iter().copied() {
            if let Some(summary) = preview_for(&ctx.attacker_skills.ultimate, target) {
                scored_actions.push(PreviewScoredAction {
                    intent: build_ultimate_intent(ctx.attacker_id, target),
                    damage: summary.total_damage,
                    action_rank: 2,
                    target_id: target,
                    skill_rank: 0,
                });
            }
        }
    }

    scored_actions
        .into_iter()
        .max_by(compare_scored_action)
        .map(|choice| choice.intent)
        .or(Some(fallback_intent))
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
