//! Aggregated harness for the action_query domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for action affordance, cast ID propagation, commander flow, and engine legality.

#[path = "action_query/action_affordance_consumers.rs"]
mod action_affordance_consumers;
#[path = "action_query/action_affordance_query.rs"]
mod action_affordance_query;
#[path = "action_query/cast_id_propagation.rs"]
mod cast_id_propagation;
#[path = "action_query/commander_flow.rs"]
mod commander_flow;
#[path = "action_query/engine_legality_integration.rs"]
mod engine_legality_integration;
#[path = "action_query/out_of_turn_burst_seam.rs"]
mod out_of_turn_burst_seam;
#[path = "action_query/combat_panel_skill_book_seam.rs"]
mod combat_panel_skill_book_seam;
