pub mod av;
pub mod helpers;
pub mod resistance;
pub mod speed;
pub mod turn_order;
pub mod types;

// Re-export extracted types so external consumers keep working.
pub use types::{ActionIntent, EnemyTurnRequestQueue};
pub(crate) use types::ResolveActorsQuery;

// Re-export extracted helpers so sibling modules (`pipeline`, `tests`) keep working.
pub(crate) use helpers::{emit_combat_beat, emit_combat_event, emit_kernel_transition, set_phase};

mod advance;
mod enemy_turn;
mod finalize;
mod resolve;

pub use advance::advance_turn_system;
pub use enemy_turn::resolve_enemy_turn_action_system;
pub use finalize::{apply_av_ops_system, check_victory_system};
pub use resolve::resolve_action_system;

mod pipeline;

pub(crate) use pipeline::{step_app, step_declaration};

#[cfg(test)]
mod tests;
