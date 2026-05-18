use crate::combat::energy::EnergyGainSource;
use crate::combat::kit::UnitSkills;
use crate::combat::state::CombatPhase;
use crate::combat::toughness::{exposes_toughness_affordance, visible_toughness, ToughnessView};
use crate::combat::types::{SkillId, UnitId};
use crate::data::skills_ron::{
    LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, TargetHpRule,
    TargetLife, TargetShape, TargetSide,
};

use super::types::{
    ActionAffordance, ActionQueryKind, ActionStatus, CombatQuerySnapshot, ImplementationStatus,
    ResourceAffordanceDetail, ResourceKind, ResourceStatus, TargetAffordance, TargetStatus,
    ToughnessAffordance, UnitQuerySnapshot,
};

// Consumed by tests/action_affordance_query.rs.
#[allow(dead_code)]
pub fn query_energy_cap_affordance(
    unit: &UnitQuerySnapshot,
    source: EnergyGainSource,
    requested: i32,
) -> ResourceAffordanceDetail {
    let cap = match source {
        EnergyGainSource::SecondaryAction => 10,
        EnergyGainSource::External => 30,
    };
    let used = match source {
        EnergyGainSource::SecondaryAction => unit.energy_secondary_gained,
        EnergyGainSource::External => unit.energy_external_gained,
    };
    let current = (cap - used).max(0);

    let status = if current >= requested {
        ResourceStatus::Enabled
    } else {
        ResourceStatus::Disabled {
            reason: LegalityReasonCode::EnergyCapReached,
        }
    };

    ResourceAffordanceDetail {
        kind: ResourceKind::EnergyCap,
        status,
        current: Some(current),
        required: Some(requested),
    }
}

fn snapshot_units(snapshot: &CombatQuerySnapshot) -> Vec<UnitQuerySnapshot> {
    if !snapshot.units.is_empty() {
        return snapshot.units.clone();
    }

    let mut units = vec![snapshot.acting_unit.clone()];
    if let Some(target_unit) = &snapshot.target_unit {
        if target_unit.id != snapshot.acting_unit.id {
            units.push(target_unit.clone());
        }
    }
    units
}

fn resolve_unit(snapshot: &CombatQuerySnapshot, unit_id: UnitId) -> Option<UnitQuerySnapshot> {
    snapshot_units(snapshot)
        .into_iter()
        .find(|unit| unit.id == unit_id)
}

fn implementation_status(skill_def: &SkillDef) -> ImplementationStatus {
    match &skill_def.implementation {
        SkillImplementation::Implemented => ImplementationStatus::Implemented,
        SkillImplementation::Deferred { reason } => ImplementationStatus::Deferred {
            reason: reason.clone(),
        },
        SkillImplementation::Hidden { reason } => ImplementationStatus::Hidden {
            reason: reason.clone(),
        },
    }
}

fn target_status_for_unit(
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn enabled_target_ids(affordance: &ActionAffordance<'_>) -> Vec<UnitId> {
    affordance
        .targets
        .iter()
        .filter_map(|(id, target)| matches!(target.status, TargetStatus::Enabled).then_some(*id))
        .collect()
}

// Consumed by tests/action_affordance_consumers.rs.
#[allow(dead_code)]
pub fn first_enabled_target_id(affordance: &ActionAffordance<'_>) -> Option<UnitId> {
    enabled_target_ids(affordance).into_iter().next()
}

fn kit_has_skill(kit: &UnitSkills, skill_id: &SkillId) -> bool {
    kit.basic == *skill_id
        || kit.ultimate == *skill_id
        || kit.skills.iter().any(|candidate| candidate == skill_id)
}

fn resolve_action_skill<'a>(
    snapshot: &'a CombatQuerySnapshot,
    skill_book: &'a SkillBook,
    actor_id: UnitId,
    kind: &ActionQueryKind<'_>,
) -> Result<(UnitQuerySnapshot, &'a SkillDef), LegalityReasonCode> {
    let actor = resolve_unit(snapshot, actor_id).ok_or(LegalityReasonCode::MissingSkill)?;
    let kit = actor
        .skills
        .as_ref()
        .ok_or(LegalityReasonCode::MissingSkill)?;

    let skill_id = match kind {
        ActionQueryKind::Basic => &kit.basic,
        ActionQueryKind::Skill(skill_id) => {
            if !kit_has_skill(kit, skill_id) {
                return Err(LegalityReasonCode::MissingSkill);
            }
            skill_id
        }
        ActionQueryKind::Ultimate => &kit.ultimate,
    };

    if !kit_has_skill(kit, skill_id) {
        return Err(LegalityReasonCode::MissingSkill);
    }

    let skill_def = skill_book
        .0
        .iter()
        .find(|skill| &skill.id == skill_id)
        .ok_or(LegalityReasonCode::MissingSkill)?;

    Ok((actor, skill_def))
}

// Called from build_resource_details -> query_action_affordance which is consumed by tests.
#[allow(dead_code)]
fn resource_detail_status(
    kind: ResourceKind,
    current: i32,
    required: i32,
) -> ResourceAffordanceDetail {
    let status = if current >= required {
        ResourceStatus::Enabled
    } else {
        ResourceStatus::Disabled {
            reason: match kind {
                ResourceKind::Sp => LegalityReasonCode::SpShortfall,
                ResourceKind::Ultimate => LegalityReasonCode::UltimateNotReady,
                ResourceKind::EnergyCap => LegalityReasonCode::EnergyCapReached,
            },
        }
    };

    ResourceAffordanceDetail {
        kind,
        status,
        current: Some(current),
        required: Some(required),
    }
}

// Called from query_action_affordance which is consumed by tests/action_affordance_query.rs.
#[allow(dead_code)]
fn build_resource_details(
    actor: &UnitQuerySnapshot,
    skill_def: &SkillDef,
    _kind: &ActionQueryKind<'_>,
    implementation: &ImplementationStatus,
) -> Vec<ResourceAffordanceDetail> {
    match implementation {
        ImplementationStatus::Hidden { reason } => vec![
            ResourceAffordanceDetail {
                kind: ResourceKind::Sp,
                status: ResourceStatus::Hidden {
                    reason: reason.clone(),
                },
                current: None,
                required: None,
            },
            ResourceAffordanceDetail {
                kind: ResourceKind::Ultimate,
                status: ResourceStatus::Hidden {
                    reason: reason.clone(),
                },
                current: None,
                required: None,
            },
        ],
        ImplementationStatus::Deferred { reason } => vec![
            ResourceAffordanceDetail {
                kind: ResourceKind::Sp,
                status: ResourceStatus::Deferred {
                    reason: reason.clone(),
                },
                current: None,
                required: None,
            },
            ResourceAffordanceDetail {
                kind: ResourceKind::Ultimate,
                status: ResourceStatus::Deferred {
                    reason: reason.clone(),
                },
                current: None,
                required: None,
            },
        ],
        ImplementationStatus::Implemented => {
            vec![
                resource_detail_status(ResourceKind::Sp, actor.sp, skill_def.sp_cost),
                resource_detail_status(
                    ResourceKind::Ultimate,
                    actor.ultimate_current,
                    actor.ultimate_trigger,
                ),
            ]
        }
    }
}

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
#[allow(dead_code)]
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
