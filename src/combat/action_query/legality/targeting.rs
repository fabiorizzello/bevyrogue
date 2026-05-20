use crate::combat::toughness::{ToughnessView, exposes_toughness_affordance, visible_toughness};
use crate::combat::types::UnitId;
use crate::data::skills_ron::{
    LegalityReasonCode, SelfTargetRule, SkillDef, SkillImplementation, TargetHpRule, TargetLife,
    TargetShape, TargetSide,
};

use super::super::types::{
    ActionAffordance, CombatQuerySnapshot, TargetAffordance, TargetStatus, ToughnessAffordance,
    UnitQuerySnapshot,
};
use super::shared::{resolve_unit, snapshot_units};

pub(super) fn target_status_for_unit(
    snapshot: &CombatQuerySnapshot,
    actor_id: UnitId,
    skill_def: &SkillDef,
    target_id: UnitId,
) -> TargetStatus {
    let actor = match resolve_unit(snapshot, actor_id) {
        Some(actor) => actor,
        None => {
            return TargetStatus::Disabled {
                reason: LegalityReasonCode::NotActiveUnit,
            };
        }
    };

    match &skill_def.implementation {
        SkillImplementation::Deferred { reason } => {
            return TargetStatus::Deferred {
                reason: reason.clone(),
            };
        }
        SkillImplementation::Hidden { reason } => {
            return TargetStatus::Hidden {
                reason: reason.clone(),
            };
        }
        SkillImplementation::Implemented => {}
    }

    if !matches!(
        skill_def.targeting.shape,
        TargetShape::Single
            | TargetShape::Blast
            | TargetShape::AllEnemies
            | TargetShape::Bounce { .. }
    ) {
        return TargetStatus::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape,
        };
    }

    let Some(target) = resolve_unit(snapshot, target_id) else {
        return TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetNotFound,
        };
    };

    if target.is_commander {
        return TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetIsCommander,
        };
    }

    if actor_id == target_id && matches!(skill_def.targeting.self_rule, SelfTargetRule::Forbid) {
        return TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetIsSelf,
        };
    }

    match skill_def.targeting.side {
        TargetSide::Any => {}
        TargetSide::Ally if target.team != actor.team => {
            return TargetStatus::Disabled {
                reason: LegalityReasonCode::WrongSide,
            };
        }
        TargetSide::Enemy if target.team == actor.team => {
            return TargetStatus::Disabled {
                reason: LegalityReasonCode::WrongSide,
            };
        }
        TargetSide::Ally | TargetSide::Enemy => {}
    }

    match skill_def.targeting.life {
        TargetLife::Any => {}
        TargetLife::Alive if target.is_ko => {
            return TargetStatus::Disabled {
                reason: LegalityReasonCode::TargetKo,
            };
        }
        TargetLife::Ko if !target.is_ko => {
            return TargetStatus::Disabled {
                reason: LegalityReasonCode::TargetNotKo,
            };
        }
        TargetLife::Alive | TargetLife::Ko => {}
    }

    if matches!(skill_def.targeting.target_hp_rule, TargetHpRule::Damaged)
        && target.hp_current >= target.hp_max
    {
        return TargetStatus::Disabled {
            reason: LegalityReasonCode::TargetFullHp,
        };
    }

    TargetStatus::Enabled
}

// Called from query_target_affordance which is consumed by tests/action_affordance_query.rs.
fn target_toughness_affordance(
    skill_def: &SkillDef,
    target: &UnitQuerySnapshot,
) -> (
    ToughnessAffordance,
    Option<ToughnessView>,
    Option<LegalityReasonCode>,
) {
    match &skill_def.implementation {
        SkillImplementation::Implemented => {
            if !exposes_toughness_affordance(target.team, target.toughness.as_ref()) {
                return (
                    ToughnessAffordance::Hidden,
                    None,
                    Some(LegalityReasonCode::ToughnessEnemyOnly),
                );
            }

            match visible_toughness(target.team, target.toughness.as_ref()) {
                Some(view) => (ToughnessAffordance::Visible, Some(view), None),
                None => (
                    ToughnessAffordance::Hidden,
                    None,
                    Some(LegalityReasonCode::ToughnessEnemyOnly),
                ),
            }
        }
        SkillImplementation::Deferred { reason } | SkillImplementation::Hidden { reason } => {
            (ToughnessAffordance::Hidden, None, Some(reason.clone()))
        }
    }
}

// Consumed by tests/action_affordance_query.rs.
pub fn query_target_affordance(
    snapshot: &CombatQuerySnapshot,
    actor_id: UnitId,
    skill_def: &SkillDef,
    target_id: UnitId,
) -> TargetAffordance {
    let status = target_status_for_unit(snapshot, actor_id, skill_def, target_id);
    let (toughness, toughness_view, toughness_reason) = resolve_unit(snapshot, target_id)
        .map(|target| target_toughness_affordance(skill_def, &target))
        .unwrap_or((
            ToughnessAffordance::Hidden,
            None,
            Some(LegalityReasonCode::TargetNotFound),
        ));

    TargetAffordance {
        status,
        toughness,
        toughness_view,
        toughness_reason,
    }
}

// Consumed by tests/action_affordance_query.rs.
pub fn query_all_target_affordances(
    snapshot: &CombatQuerySnapshot,
    actor_id: UnitId,
    skill_def: &SkillDef,
) -> Vec<(UnitId, TargetAffordance)> {
    snapshot_units(snapshot)
        .into_iter()
        .map(|unit| {
            let affordance = query_target_affordance(snapshot, actor_id, skill_def, unit.id);
            (unit.id, affordance)
        })
        .collect()
}

// Consumed by tests/action_affordance_consumers.rs.
pub fn enabled_target_ids(affordance: &ActionAffordance<'_>) -> Vec<UnitId> {
    affordance
        .targets
        .iter()
        .filter_map(|(id, target)| matches!(target.status, TargetStatus::Enabled).then_some(*id))
        .collect()
}

// Consumed by tests/action_affordance_consumers.rs.
pub fn first_enabled_target_id(affordance: &ActionAffordance<'_>) -> Option<UnitId> {
    enabled_target_ids(affordance).into_iter().next()
}
