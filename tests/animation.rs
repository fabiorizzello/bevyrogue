//! Aggregated harness for the animation domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for animation graphs, clips, player FSM, and asset validation.

#[path = "animation/agumon_sharp_claws_asset.rs"]
mod agumon_sharp_claws_asset;
#[path = "animation/anim_gameplay_command_forbidden.rs"]
mod anim_gameplay_command_forbidden;
#[path = "animation/anim_graph_asset.rs"]
mod anim_graph_asset;
#[path = "animation/anim_graph_input_purity.rs"]
mod anim_graph_input_purity;
#[path = "animation/anim_graph_parse.rs"]
mod anim_graph_parse;
#[path = "animation/anim_player_fsm.rs"]
mod anim_player_fsm;
#[path = "animation/anim_registry_failure_visibility.rs"]
mod anim_registry_failure_visibility;
#[path = "animation/anim_stance_asset.rs"]
mod anim_stance_asset;
#[path = "animation/anim_validation.rs"]
mod anim_validation;
#[path = "animation/atlas_binding.rs"]
mod atlas_binding;
#[path = "animation/clip_atlas_parity.rs"]
mod clip_atlas_parity;
#[path = "animation/skill_graph_mapping_extensibility.rs"]
mod skill_graph_mapping_extensibility;
#[path = "animation/vfx_handle_seam.rs"]
mod vfx_handle_seam;
#[path = "animation/vfx_spawn_descriptor.rs"]
mod vfx_spawn_descriptor;
