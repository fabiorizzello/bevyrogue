mod legality;
mod types;

pub use legality::{
    enabled_target_ids, first_enabled_target_id, query_action_affordance,
    query_all_target_affordances, query_energy_cap_affordance, query_intent_legality,
    query_target_affordance,
};
pub use types::{
    build_snapshot_from_ecs, build_snapshot_from_ecs_with_sp, ActionAffordance, ActionQueryKind,
    ActionStatus, CombatQuerySnapshot, ImplementationStatus, ResourceAffordanceDetail, ResourceKind,
    ResourceStatus, TargetAffordance, TargetStatus, ToughnessAffordance, UnitQuerySnapshot,
};
