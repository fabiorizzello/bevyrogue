use crate::combat::state::CombatPhase;
use crate::combat::types::UnitId;
use crate::data::skills_ron::{LegalityReasonCode, SkillBook, SkillDef};

use super::super::types::{
    ActionAffordance, ActionQueryKind, ActionStatus, CombatQuerySnapshot, ImplementationStatus,
    ResourceStatus, TargetAffordance, TargetStatus, ToughnessAffordance, UnitQuerySnapshot,
};
use super::resources::build_resource_details;
use super::shared::{implementation_status, resolve_action_skill};
use super::targeting::{query_all_target_affordances, target_status_for_unit};

fn aggregate_target_status(
    targets: &[(UnitId, TargetAffordance)],
    implementation: &ImplementationStatus,
) -> TargetStatus {
    match implementation {
        ImplementationStatus::Hidden { reason } => TargetStatus::Hidden {
            reason: reason.clone(),
        },
        ImplementationStatus::Deferred { reason } => TargetStatus::Deferred {
            reason: reason.clone(),
        },
        ImplementationStatus::Implemented => {
            if targets
                .iter()
                .any(|(_, affordance)| matches!(affordance.status, TargetStatus::Enabled))
            {
                TargetStatus::Enabled
            } else {
                TargetStatus::Disabled {
                    reason: LegalityReasonCode::NoValidTargets,
                }
            }
        }
    }
}

fn action_and_resource_status_for_snapshot(
    snapshot: &CombatQuerySnapshot,
    actor: &UnitQuerySnapshot,
    skill_def: &SkillDef,
    kind: &ActionQueryKind<'_>,
    targets: &[(UnitId, TargetAffordance)],
) -> (ActionStatus, ResourceStatus) {
    if !actor.is_active {
        return (
            ActionStatus::Disabled {
                reason: LegalityReasonCode::NotActiveUnit,
            },
            ResourceStatus::Disabled {
                reason: LegalityReasonCode::NotActiveUnit,
            },
        );
    }

    if snapshot.phase != CombatPhase::WaitingAction {
        return (
            ActionStatus::Disabled {
                reason: LegalityReasonCode::WrongPhase,
            },
            ResourceStatus::Disabled {
                reason: LegalityReasonCode::WrongPhase,
            },
        );
    }

    if actor.is_ko {
        return (
            ActionStatus::Disabled {
                reason: LegalityReasonCode::AttackerKo,
            },
            ResourceStatus::Disabled {
                reason: LegalityReasonCode::AttackerKo,
            },
        );
    }

    if actor.is_stunned {
        return (
            ActionStatus::Disabled {
                reason: LegalityReasonCode::AttackerStunned,
            },
            ResourceStatus::Disabled {
                reason: LegalityReasonCode::AttackerStunned,
            },
        );
    }

    match kind {
        ActionQueryKind::Ultimate => {
            if actor.sp < skill_def.sp_cost {
                return (
                    ActionStatus::Disabled {
                        reason: LegalityReasonCode::SpShortfall,
                    },
                    ResourceStatus::Disabled {
                        reason: LegalityReasonCode::SpShortfall,
                    },
                );
            }

            if !actor.ultimate_ready || actor.ultimate_current < actor.ultimate_trigger {
                return (
                    ActionStatus::Disabled {
                        reason: LegalityReasonCode::UltimateNotReady,
                    },
                    ResourceStatus::Disabled {
                        reason: LegalityReasonCode::UltimateNotReady,
                    },
                );
            }
        }
        ActionQueryKind::Basic | ActionQueryKind::Skill(_) => {
            if actor.sp < skill_def.sp_cost {
                return (
                    ActionStatus::Disabled {
                        reason: LegalityReasonCode::SpShortfall,
                    },
                    ResourceStatus::Disabled {
                        reason: LegalityReasonCode::SpShortfall,
                    },
                );
            }
        }
    }

    if matches!(
        aggregate_target_status(targets, &ImplementationStatus::Implemented),
        TargetStatus::Disabled {
            reason: LegalityReasonCode::NoValidTargets
        }
    ) {
        return (
            ActionStatus::Disabled {
                reason: LegalityReasonCode::NoValidTargets,
            },
            ResourceStatus::Disabled {
                reason: LegalityReasonCode::NoValidTargets,
            },
        );
    }

    (ActionStatus::Enabled, ResourceStatus::Enabled)
}

// Consumed by tests/action_affordance_query.rs and tests/action_affordance_consumers.rs.
pub fn query_action_affordance<'a>(
    snapshot: &CombatQuerySnapshot,
    skill_book: &SkillBook,
    actor_id: UnitId,
    kind: ActionQueryKind<'a>,
) -> ActionAffordance<'a> {
    let Ok((actor, skill_def)) = resolve_action_skill(snapshot, skill_book, actor_id, &kind) else {
        let reason = LegalityReasonCode::MissingSkill;
        return ActionAffordance {
            kind,
            action: ActionStatus::Disabled {
                reason: reason.clone(),
            },
            target: TargetStatus::Disabled {
                reason: reason.clone(),
            },
            targets: vec![],
            resource: ResourceStatus::Disabled {
                reason: reason.clone(),
            },
            resource_details: vec![],
            implementation: ImplementationStatus::Implemented,
            toughness: ToughnessAffordance::Hidden,
        };
    };

    let targets = query_all_target_affordances(snapshot, actor_id, skill_def);
    let target = aggregate_target_status(&targets, &ImplementationStatus::Implemented);
    let implementation = implementation_status(skill_def);
    let resource_details = build_resource_details(&actor, skill_def, &kind, &implementation);
    let (action, resource) =
        action_and_resource_status_for_snapshot(snapshot, &actor, skill_def, &kind, &targets);
    let toughness = targets
        .iter()
        .any(|(_, affordance)| matches!(affordance.toughness, ToughnessAffordance::Visible));

    ActionAffordance {
        kind,
        action,
        target,
        targets,
        resource,
        resource_details,
        implementation,
        toughness: if toughness {
            ToughnessAffordance::Visible
        } else {
            ToughnessAffordance::Hidden
        },
    }
}

/// Validates a specific selected intent (actor + action kind + target) against the existing
/// query infrastructure and returns Result<(), LegalityReasonCode>.
///
/// Priority: missing skill > implementation > actor/resource > selected target.
pub fn query_intent_legality(
    snapshot: &CombatQuerySnapshot,
    skill_book: &SkillBook,
    actor_id: UnitId,
    kind: &ActionQueryKind<'_>,
    target_id: UnitId,
) -> Result<(), LegalityReasonCode> {
    // 1. Resolve skill
    let (actor, skill_def) = resolve_action_skill(snapshot, skill_book, actor_id, kind)?;

    // 2. Implementation status
    match implementation_status(skill_def) {
        ImplementationStatus::Implemented => {}
        ImplementationStatus::Deferred { reason } | ImplementationStatus::Hidden { reason } => {
            return Err(reason);
        }
    }

    // 3. Actor/Phase/Resource status
    // We use an empty targets list to let action_and_resource_status_for_snapshot perform actor/phase/resource checks.
    // It will return NoValidTargets if those pass, which we ignore to proceed to the specific target check.
    let (action_status, _) =
        action_and_resource_status_for_snapshot(snapshot, &actor, skill_def, kind, &[]);
    match action_status {
        ActionStatus::Enabled => {}
        ActionStatus::Disabled { reason } => {
            if reason != LegalityReasonCode::NoValidTargets {
                return Err(reason);
            }
        }
        ActionStatus::Deferred { reason } | ActionStatus::Hidden { reason } => {
            return Err(reason);
        }
    }

    // 4. Specific target check
    match target_status_for_unit(snapshot, actor_id, skill_def, target_id) {
        TargetStatus::Enabled => Ok(()),
        TargetStatus::Disabled { reason }
        | TargetStatus::Deferred { reason }
        | TargetStatus::Hidden { reason } => Err(reason),
    }
}
