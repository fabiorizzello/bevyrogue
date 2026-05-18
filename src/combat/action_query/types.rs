use crate::combat::energy::{Energy, RoundEnergyTracker};
use crate::combat::kit::UnitSkills;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, CombatState};
use crate::combat::team::Team;
use crate::combat::toughness::{Toughness, ToughnessView};
use crate::combat::turn_order::TurnOrder;
use crate::combat::types::{SkillId, UnitId};
use crate::combat::ultimate::UltimateCharge;
use crate::combat::unit::Unit;
use crate::data::skills_ron::LegalityReasonCode;

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
        Option<&crate::combat::counterplay::EnemyCounterplayKit>,
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
        Option<&crate::combat::counterplay::EnemyCounterplayKit>,
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
        _enemy_counterplay,
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
        // Deferred/Hidden variants consumed by tests/action_affordance_query.rs and consumers.
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

// Deferred/Hidden consumed by tests/action_affordance_query.rs and action_affordance_consumers.rs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImplementationStatus {
    Implemented,
    Deferred { reason: LegalityReasonCode },
    Hidden { reason: LegalityReasonCode },
}

// Hidden/Visible consumed by tests/action_affordance_query.rs.
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

// Consumed by tests/action_affordance_consumers.rs and action_affordance_query.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Sp,
    Ultimate,
    EnergyCap,
}

// Consumed by tests/action_affordance_query.rs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAffordanceDetail {
    pub kind: ResourceKind,
    pub status: ResourceStatus,
    pub current: Option<i32>,
    pub required: Option<i32>,
}

// Consumed by tests/action_affordance_consumers.rs via the public surface.
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
