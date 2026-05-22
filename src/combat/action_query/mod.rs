mod legality;
mod types;

pub use legality::{
    enabled_target_ids, first_enabled_target_id, query_action_affordance,
    query_all_target_affordances, query_intent_legality, query_target_affordance,
};
pub use types::{
    ActionAffordance, ActionQueryKind, ActionStatus, CombatQuerySnapshot, ImplementationStatus,
    ResourceAffordanceDetail, ResourceKind, ResourceStatus, TargetAffordance, TargetStatus,
    ToughnessAffordance, UnitQuerySnapshot, build_snapshot_from_ecs,
    build_snapshot_from_ecs_with_sp, mark_unit_active,
};
