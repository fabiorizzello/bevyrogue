//! Aggregated harness for the preview_ai domain. See .gsd/KNOWLEDGE.md R003.
//!
//! Hosts tests for enemy AI decision routing, skill preview, and presentation metadata.

#[path = "common/mod.rs"]
mod common;

#[path = "preview_ai/enemy_ai.rs"]
mod enemy_ai;
#[path = "preview_ai/enemy_ai_internals.rs"]
mod enemy_ai_internals;
#[path = "preview_ai/enemy_ai_preview.rs"]
mod enemy_ai_preview;
#[path = "preview_ai/presentation_metadata_boundary.rs"]
mod presentation_metadata_boundary;
#[path = "preview_ai/scenario_ttk.rs"]
mod scenario_ttk;
#[path = "preview_ai/skill_preview.rs"]
mod skill_preview;
