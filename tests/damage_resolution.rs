//! Aggregated harness for the damage_resolution domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for damage calculation, block reactions, DR pipeline, and intent application.

#[path = "common/mod.rs"]
mod common;

#[path = "damage_resolution/block_reaction_pipeline.rs"]
mod block_reaction_pipeline;
#[path = "damage_resolution/combat_coherence.rs"]
mod combat_coherence;
#[path = "damage_resolution/combat_damage_edge.rs"]
mod combat_damage_edge;
#[path = "damage_resolution/combat_damage_matrix.rs"]
mod combat_damage_matrix;
#[path = "damage_resolution/combat_resolution_apply.rs"]
mod combat_resolution_apply;
#[path = "damage_resolution/damage_breakdown_log.rs"]
mod damage_breakdown_log;
#[path = "damage_resolution/dr_pipeline.rs"]
mod dr_pipeline;
#[path = "damage_resolution/intent_applier_canary.rs"]
mod intent_applier_canary;
