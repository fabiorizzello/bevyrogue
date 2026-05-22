mod action;
mod resources;
mod shared;
mod targeting;

pub use action::{query_action_affordance, query_intent_legality};
pub use targeting::{
    enabled_target_ids, first_enabled_target_id, query_all_target_affordances,
    query_target_affordance,
};
