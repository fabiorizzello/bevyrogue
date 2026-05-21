//! Aggregated harness for the target_shape domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for target shape resolution, bounce, AoE, blast, and truthfulness.

#[path = "common/mod.rs"]
mod common;

#[path = "target_shape/combat_resolution_bounce.rs"]
mod combat_resolution_bounce;
#[path = "target_shape/combat_resolution_targets.rs"]
mod combat_resolution_targets;
#[path = "target_shape/target_shape_aoe_and_blast.rs"]
mod target_shape_aoe_and_blast;
#[path = "target_shape/target_shape_bounce_chain.rs"]
mod target_shape_bounce_chain;
#[path = "target_shape/target_shape_truthfulness.rs"]
mod target_shape_truthfulness;
