//! Aggregated harness for the windowed_only domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for windowed-only UI features (phase strip, preview cache).

#[path = "windowed_only/frame_time_soak.rs"]
mod frame_time_soak;
#[path = "windowed_only/phase_strip_readonly.rs"]
mod phase_strip_readonly;
#[path = "windowed_only/vfx_asset_impact_render.rs"]
mod vfx_asset_impact_render;
#[path = "windowed_only/vfx_rendering_acceptance.rs"]
mod vfx_rendering_acceptance;
#[path = "windowed_only/windowed_hud_hp_bar.rs"]
mod windowed_hud_hp_bar;
#[path = "windowed_only/windowed_preview_cache.rs"]
mod windowed_preview_cache;
#[path = "windowed_only/windowed_target_hurt.rs"]
mod windowed_target_hurt;
#[path = "windowed_only/windowed_twin_core_badge.rs"]
mod windowed_twin_core_badge;
