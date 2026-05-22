use crate::data::skills_ron::{LegalityReasonCode, SkillDef};

use super::super::types::{
    ActionQueryKind, ImplementationStatus, ResourceAffordanceDetail, ResourceKind, ResourceStatus,
    UnitQuerySnapshot,
};
use super::shared::ult_readiness_from_snapshot;

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
            // Derive Ultimate detail from the unified gauge seam so energy-backed units
            // (e.g. Agumon) report Energy values; legacy units stay on UltimateCharge.
            let (ult_current, ult_trigger, _) = ult_readiness_from_snapshot(actor);
            vec![
                resource_detail_status(ResourceKind::Sp, actor.sp, skill_def.sp_cost),
                resource_detail_status(ResourceKind::Ultimate, ult_current, ult_trigger),
            ]
        }
    }
}
