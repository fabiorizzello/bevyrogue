use crate::combat::counterplay::LegalityReasonCode as CounterplayLegalityReasonCode;
use crate::combat::counterplay::{
    ChargedAttackDeclaration, EnemyCounterplayKind, EnemyTraitDeclaration,
};
use crate::combat::energy::EnergyGainSource;
use crate::combat::energy::{Energy, RoundEnergyTracker};
use crate::combat::kit::UnitSkills;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, CombatState};
use crate::combat::team::Team;
use crate::combat::toughness::{
    Toughness, ToughnessView, exposes_toughness_affordance, visible_toughness,
};
use crate::combat::turn_order::TurnOrder;
use crate::combat::types::{SkillId, UnitId};
use crate::combat::ultimate::UltimateCharge;
use crate::combat::unit::Unit;
use crate::data::skills_ron::{
    LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, TargetHpRule,
    TargetLife, TargetShape, TargetSide,
};
use crate::data::units_ron::EnemyCounterplayStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatQuerySnapshot {
    pub phase: CombatPhase,
    pub acting_unit: UnitQuerySnapshot,
    pub target_unit: Option<UnitQuerySnapshot>,
    pub units: Vec<UnitQuerySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitQuerySnapshot {
    pub id: UnitId,
    pub team: Team,
    pub is_active: bool,
    pub is_ko: bool,
    pub is_stunned: bool,
    pub is_commander: bool,
    pub hp_current: i32,
    pub hp_max: i32,
    pub sp: i32,
    pub ultimate_current: i32,
    pub ultimate_trigger: i32,
    pub ultimate_ready: bool,
    pub energy: i32,
    pub energy_secondary_gained: i32,
    pub energy_external_gained: i32,
    pub skills: Option<UnitSkills>,
    pub toughness: Option<Toughness>,
    pub enemy_traits: Vec<EnemyTraitDeclaration>,
    pub charged_attack: Option<ChargedAttackDeclaration>,
}

impl Default for UnitQuerySnapshot {
    fn default() -> Self {
        Self {
            id: UnitId(0),
            team: Team::Ally,
            is_active: false,
            is_ko: false,
            is_stunned: false,
            is_commander: false,
            hp_current: 0,
            hp_max: 0,
            sp: 0,
            ultimate_current: 0,
            ultimate_trigger: 100,
            ultimate_ready: false,
            energy: 0,
            energy_secondary_gained: 0,
            energy_external_gained: 0,
            skills: None,
            toughness: None,
            enemy_traits: vec![],
            charged_attack: None,
        }
    }
}

pub fn build_snapshot_from_ecs(
    state: &CombatState,
    turn_order: &TurnOrder,
    _sp_pool: &SpPool,
    actor_id: UnitId,
    target_id: UnitId,
    units_data: Vec<(
        UnitId,
        Team,
        &Unit,
        Option<&UnitSkills>,
        Option<&UltimateCharge>,
        Option<&Toughness>,
        Option<&crate::combat::enemy_counterplay::EnemyCounterplayKit>,
        bool, // is_ko
        bool, // is_stunned
        bool, // is_commander
        Option<&Energy>,
        Option<&RoundEnergyTracker>,
    )>,
) -> CombatQuerySnapshot {
    build_snapshot_from_ecs_with_sp(state, turn_order, i32::MAX, actor_id, target_id, units_data)
}

/// Builds a snapshot for UI/CLI affordance consumers using the provided SP value.
///
/// The engine-facing `build_snapshot_from_ecs()` wrapper intentionally keeps the
/// SP-bypass path intact for S06 parity checks; UI/CLI callers can use this helper
/// to reflect the real `SpPool.current` value in preflight affordances.
pub fn build_snapshot_from_ecs_with_sp(
    state: &CombatState,
    turn_order: &TurnOrder,
    sp_current: i32,
    actor_id: UnitId,
    target_id: UnitId,
    units_data: Vec<(
        UnitId,
        Team,
        &Unit,
        Option<&UnitSkills>,
        Option<&UltimateCharge>,
        Option<&Toughness>,
        Option<&crate::combat::enemy_counterplay::EnemyCounterplayKit>,
        bool, // is_ko
        bool, // is_stunned
        bool, // is_commander
        Option<&Energy>,
        Option<&RoundEnergyTracker>,
    )>,
) -> CombatQuerySnapshot {
    let mut units = Vec::new();

    for (
        id,
        team,
        unit,
        skills,
        ult,
        toughness,
        enemy_counterplay,
        is_ko,
        is_stunned,
        is_commander,
        energy,
        energy_tracker,
    ) in units_data
    {
        let is_active = if let Some(active) = turn_order.active_unit {
            id == active
        } else {
            id == actor_id
        };

        units.push(UnitQuerySnapshot {
            id,
            team,
            is_active,
            is_ko,
            is_stunned,
            is_commander,
            hp_current: unit.hp_current,
            hp_max: unit.hp_max,
            sp: sp_current,
            ultimate_current: ult.map(|u| u.current).unwrap_or(0),
            ultimate_trigger: ult.map(|u| u.trigger).unwrap_or(100),
            ultimate_ready: ult.map(|u| u.ready()).unwrap_or(false),
            energy: energy.map(|e| e.current).unwrap_or(0),
            energy_secondary_gained: energy_tracker.map(|t| t.secondary_gained).unwrap_or(0),
            energy_external_gained: energy_tracker.map(|t| t.external_gained).unwrap_or(0),
            skills: skills.cloned(),
            toughness: toughness.cloned(),
            enemy_traits: enemy_counterplay
                .map(|kit| kit.enemy_traits.clone())
                .unwrap_or_default(),
            charged_attack: enemy_counterplay.and_then(|kit| kit.charged_attack.clone()),
        });
    }

    let acting_unit = units
        .iter()
        .find(|u| u.id == actor_id)
        .cloned()
        .unwrap_or_else(|| {
            // Fallback for missing actor (should be caught by query)
            UnitQuerySnapshot {
                id: actor_id,
                team: Team::Ally,
                is_active: true,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                hp_current: 100,
                hp_max: 100,
                sp: sp_current,
                ultimate_current: 0,
                ultimate_trigger: 100,
                ultimate_ready: false,
                energy: 0,
                energy_secondary_gained: 0,
                energy_external_gained: 0,
                skills: None,
                toughness: None,
                enemy_traits: vec![],
                charged_attack: None,
            }
        });

    let target_unit = units.iter().find(|u| u.id == target_id).cloned();

    CombatQuerySnapshot {
        phase: state.phase,
        acting_unit,
        target_unit,
        units,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionQueryKind<'a> {
    Basic,
    Skill(&'a SkillId),
    Ultimate,
}

macro_rules! status_enum {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            Enabled,
            Disabled { reason: LegalityReasonCode },
            Deferred { reason: LegalityReasonCode },
            Hidden { reason: LegalityReasonCode },
        }
    };
}

status_enum!(ActionStatus);
status_enum!(TargetStatus);
status_enum!(ResourceStatus);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImplementationStatus {
    Implemented,
    Deferred { reason: LegalityReasonCode },
    Hidden { reason: LegalityReasonCode },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToughnessAffordance {
    Hidden,
    Visible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetAffordance {
    pub status: TargetStatus,
    pub toughness: ToughnessAffordance,
    pub toughness_view: Option<ToughnessView>,
    pub toughness_reason: Option<LegalityReasonCode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Sp,
    Ultimate,
    TamerGauge,
    TamerCommand,
    ChargedTelegraph,
    EnemyTrait,
    EnergyCap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAffordanceDetail {
    pub kind: ResourceKind,
    pub status: ResourceStatus,
    pub current: Option<i32>,
    pub required: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionAffordance<'a> {
    pub kind: ActionQueryKind<'a>,
    pub action: ActionStatus,
    pub target: TargetStatus,
    pub targets: Vec<(UnitId, TargetAffordance)>,
    pub resource: ResourceStatus,
    pub resource_details: Vec<ResourceAffordanceDetail>,
    pub implementation: ImplementationStatus,
    pub toughness: ToughnessAffordance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnemyTraitAffordance {
    pub kind: EnemyCounterplayKind,
    pub implementation: ImplementationStatus,
    pub resource: ResourceAffordanceDetail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChargedTelegraphAffordance {
    pub skill_id: SkillId,
    pub lead_turns: u32,
    pub implementation: ImplementationStatus,
    pub resource: ResourceAffordanceDetail,
}

fn counterplay_reason(reason: &CounterplayLegalityReasonCode) -> CounterplayLegalityReasonCode {
    *reason
}

fn enemy_trait_implementation(status: &EnemyCounterplayStatus) -> ImplementationStatus {
    match status {
        EnemyCounterplayStatus::Implemented => ImplementationStatus::Implemented,
        EnemyCounterplayStatus::Deferred { reason } => ImplementationStatus::Deferred {
            reason: counterplay_reason(reason),
        },
        EnemyCounterplayStatus::Hidden { reason } => ImplementationStatus::Hidden {
            reason: counterplay_reason(reason),
        },
    }
}

fn enemy_trait_resource_detail(status: &ImplementationStatus) -> ResourceAffordanceDetail {
    match status {
        ImplementationStatus::Implemented => ResourceAffordanceDetail {
            kind: ResourceKind::EnemyTrait,
            status: ResourceStatus::Enabled,
            current: Some(1),
            required: Some(1),
        },
        ImplementationStatus::Deferred { reason } => {
            deferred_resource_detail(ResourceKind::EnemyTrait, reason.clone())
        }
        ImplementationStatus::Hidden { reason } => {
            hidden_resource_detail(ResourceKind::EnemyTrait, reason.clone())
        }
    }
}

fn charged_telegraph_implementation(status: &EnemyCounterplayStatus) -> ImplementationStatus {
    enemy_trait_implementation(status)
}

fn charged_telegraph_resource_detail(status: &ImplementationStatus) -> ResourceAffordanceDetail {
    match status {
        ImplementationStatus::Implemented => ResourceAffordanceDetail {
            kind: ResourceKind::ChargedTelegraph,
            status: ResourceStatus::Enabled,
            current: Some(1),
            required: Some(1),
        },
        ImplementationStatus::Deferred { reason } => {
            deferred_resource_detail(ResourceKind::ChargedTelegraph, reason.clone())
        }
        ImplementationStatus::Hidden { reason } => {
            hidden_resource_detail(ResourceKind::ChargedTelegraph, reason.clone())
        }
    }
}

pub fn query_enemy_trait_affordances(unit: &UnitQuerySnapshot) -> Vec<EnemyTraitAffordance> {
    unit.enemy_traits
        .iter()
        .map(|decl| {
            let implementation = enemy_trait_implementation(&decl.status);
            let resource = enemy_trait_resource_detail(&implementation);
            EnemyTraitAffordance {
                kind: decl.kind,
                implementation,
                resource,
            }
        })
        .collect()
}

pub fn query_charged_telegraph_affordance(
    unit: &UnitQuerySnapshot,
) -> Option<ChargedTelegraphAffordance> {
    let decl = unit.charged_attack.as_ref()?;
    let implementation = charged_telegraph_implementation(&decl.status);
    let resource = charged_telegraph_resource_detail(&implementation);
    Some(ChargedTelegraphAffordance {
        skill_id: decl.skill_id.clone(),
        lead_turns: decl.lead_turns as u32,
        implementation,
        resource,
    })
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
        TargetShape::Single | TargetShape::Blast | TargetShape::AllEnemies | TargetShape::Bounce { .. }
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

/// Returns only the target ids that are enabled in the affordance snapshot.
///
/// This helper intentionally stays dumb: it filters the query output without
/// adding any extra legality rules, so CLI/windowed consumers can reuse it
/// without re-encoding KO/team/target-side logic.
pub fn enabled_target_ids(affordance: &ActionAffordance<'_>) -> Vec<UnitId> {
    affordance
        .targets
        .iter()
        .filter_map(|(id, target)| matches!(target.status, TargetStatus::Enabled).then_some(*id))
        .collect()
}

/// Returns the first enabled target id from the affordance snapshot, preserving
/// the query order for simple default-selection consumers.
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
                ResourceKind::TamerGauge => LegalityReasonCode::TamerGaugeDeferred,
                ResourceKind::TamerCommand => LegalityReasonCode::TamerCommandDeferred,
                ResourceKind::ChargedTelegraph => LegalityReasonCode::ChargedTelegraphDeferred,
                ResourceKind::EnemyTrait => LegalityReasonCode::EnemyTraitDeferred,
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

fn hidden_resource_detail(
    kind: ResourceKind,
    reason: LegalityReasonCode,
) -> ResourceAffordanceDetail {
    ResourceAffordanceDetail {
        kind,
        status: ResourceStatus::Hidden { reason },
        current: None,
        required: None,
    }
}

fn deferred_resource_detail(
    kind: ResourceKind,
    reason: LegalityReasonCode,
) -> ResourceAffordanceDetail {
    ResourceAffordanceDetail {
        kind,
        status: ResourceStatus::Deferred { reason },
        current: None,
        required: None,
    }
}

fn energy_cap_remaining(unit: &UnitQuerySnapshot, source: EnergyGainSource) -> i32 {
    let cap = match source {
        EnergyGainSource::SecondaryAction => 10,
        EnergyGainSource::External => 30,
    };

    let used = match source {
        EnergyGainSource::SecondaryAction => unit.energy_secondary_gained,
        EnergyGainSource::External => unit.energy_external_gained,
    };

    (cap - used).max(0)
}

pub fn query_energy_cap_affordance(
    unit: &UnitQuerySnapshot,
    source: EnergyGainSource,
    requested: i32,
) -> ResourceAffordanceDetail {
    let current = energy_cap_remaining(unit, source);
    if current >= requested {
        resource_detail_status(ResourceKind::EnergyCap, current, requested)
    } else {
        ResourceAffordanceDetail {
            kind: ResourceKind::EnergyCap,
            status: ResourceStatus::Disabled {
                reason: LegalityReasonCode::EnergyCapReached,
            },
            current: Some(current),
            required: Some(requested),
        }
    }
}

pub fn query_tamer_gauge_affordance() -> ResourceAffordanceDetail {
    deferred_resource_detail(
        ResourceKind::TamerGauge,
        LegalityReasonCode::TamerGaugeDeferred,
    )
}

pub fn query_tamer_command_affordances() -> Vec<ResourceAffordanceDetail> {
    [20, 50, 100]
        .into_iter()
        .map(|cost| ResourceAffordanceDetail {
            kind: ResourceKind::TamerCommand,
            status: ResourceStatus::Deferred {
                reason: LegalityReasonCode::TamerCommandDeferred,
            },
            current: None,
            required: Some(cost),
        })
        .collect()
}

fn build_resource_details(
    actor: &UnitQuerySnapshot,
    skill_def: &SkillDef,
    kind: &ActionQueryKind<'_>,
    implementation: &ImplementationStatus,
) -> Vec<ResourceAffordanceDetail> {
    match implementation {
        ImplementationStatus::Hidden { reason } => vec![
            hidden_resource_detail(ResourceKind::Sp, reason.clone()),
            hidden_resource_detail(ResourceKind::Ultimate, reason.clone()),
        ],
        ImplementationStatus::Deferred { reason } => vec![
            deferred_resource_detail(ResourceKind::Sp, reason.clone()),
            deferred_resource_detail(ResourceKind::Ultimate, reason.clone()),
        ],
        ImplementationStatus::Implemented => {
            let mut details = vec![
                resource_detail_status(ResourceKind::Sp, actor.sp, skill_def.sp_cost),
                resource_detail_status(
                    ResourceKind::Ultimate,
                    actor.ultimate_current,
                    actor.ultimate_trigger,
                ),
            ];

            if matches!(kind, ActionQueryKind::Ultimate) {
                details[1] = resource_detail_status(
                    ResourceKind::Ultimate,
                    actor.ultimate_current,
                    actor.ultimate_trigger,
                );
            }

            details
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

fn implementation_block(
    skill_def: &SkillDef,
) -> Option<(
    ActionStatus,
    ResourceStatus,
    TargetStatus,
    ImplementationStatus,
)> {
    match &skill_def.implementation {
        SkillImplementation::Deferred { reason } => Some((
            ActionStatus::Deferred {
                reason: reason.clone(),
            },
            ResourceStatus::Deferred {
                reason: reason.clone(),
            },
            TargetStatus::Deferred {
                reason: reason.clone(),
            },
            ImplementationStatus::Deferred {
                reason: reason.clone(),
            },
        )),
        SkillImplementation::Hidden { reason } => Some((
            ActionStatus::Hidden {
                reason: reason.clone(),
            },
            ResourceStatus::Hidden {
                reason: reason.clone(),
            },
            TargetStatus::Hidden {
                reason: reason.clone(),
            },
            ImplementationStatus::Hidden {
                reason: reason.clone(),
            },
        )),
        SkillImplementation::Implemented => None,
    }
}

fn missing_skill_affordance<'a>(kind: ActionQueryKind<'a>) -> ActionAffordance<'a> {
    let reason = LegalityReasonCode::MissingSkill;
    ActionAffordance {
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
    }
}

pub fn query_action_affordance<'a>(
    snapshot: &CombatQuerySnapshot,
    skill_book: &SkillBook,
    actor_id: UnitId,
    kind: ActionQueryKind<'a>,
) -> ActionAffordance<'a> {
    let Ok((actor, skill_def)) = resolve_action_skill(snapshot, skill_book, actor_id, &kind) else {
        return missing_skill_affordance(kind);
    };

    if let Some((action, resource, target, implementation)) = implementation_block(skill_def) {
        let targets = query_all_target_affordances(snapshot, actor_id, skill_def);
        let resource_details = build_resource_details(&actor, skill_def, &kind, &implementation);
        let toughness = targets
            .iter()
            .any(|(_, affordance)| matches!(affordance.toughness, ToughnessAffordance::Visible));
        return ActionAffordance {
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
        };
    }

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
