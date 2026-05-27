//! Aggregated harness for the windowed_only domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for windowed-only UI features (phase strip, preview cache).

#[path = "windowed_only/agumon_module_extraction.rs"]
mod agumon_module_extraction;
#[path = "windowed_only/digimon_sprite_cue_dispatch.rs"]
mod digimon_sprite_cue_dispatch;
#[path = "windowed_only/enoki_impact_effect_parses.rs"]
mod enoki_impact_effect_parses;
#[path = "windowed_only/enoki_impact_render.rs"]
mod enoki_impact_render;
#[path = "windowed_only/enoki_skill_effects_parse.rs"]
mod enoki_skill_effects_parse;
#[path = "windowed_only/frame_time_soak.rs"]
mod frame_time_soak;
#[path = "windowed_only/vfx_presets_parse.rs"]
mod vfx_presets_parse;
#[path = "windowed_only/renamon_extension_contract.rs"]
mod renamon_extension_contract;
#[path = "windowed_only/vfx_asset_impact_render.rs"]
mod vfx_asset_impact_render;
#[path = "windowed_only/vfx_windowed_contracts.rs"]
mod vfx_windowed_contracts;
#[path = "windowed_only/windowed_hit_feedback.rs"]
mod windowed_hit_feedback;
#[path = "windowed_only/windowed_hud_hp_bar.rs"]
mod windowed_hud_hp_bar;
#[path = "windowed_only/windowed_preview_cache.rs"]
mod windowed_preview_cache;
#[path = "windowed_only/windowed_target_hurt.rs"]
mod windowed_target_hurt;
#[path = "windowed_only/windowed_twin_core_badge.rs"]
mod windowed_twin_core_badge;
