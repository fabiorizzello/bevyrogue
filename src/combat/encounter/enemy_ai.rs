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
// hp_current/hp_max consumed by tests/enemy_ai.rs and tests/enemy_ai_preview.rs.
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
// Used in cfg(test) within enemy_ai.rs; public seam for runtime bridge.
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
