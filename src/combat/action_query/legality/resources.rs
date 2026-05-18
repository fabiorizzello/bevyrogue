use crate::combat::energy::EnergyGainSource;
use crate::data::skills_ron::{LegalityReasonCode, SkillDef};

use super::super::types::{
    ActionQueryKind, ImplementationStatus, ResourceAffordanceDetail, ResourceKind, ResourceStatus,
    UnitQuerySnapshot,
};

// Consumed by tests/action_affordance_query.rs.
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

// Called from build_resource_details -> query_action_affordance which is consumed by tests.
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
pub(super) fn build_resource_details(
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
