//! Per-Digimon presentation modules for the windowed app.
//!
//! Each Digimon owns a submodule with a `register(app)` entry point that
//! populates the generic engine registries (cues, effects, skills) with its
//! specific data. The engine systems stay generic and read from those
//! registries; the per-Digimon data and control-flow live here. Mirrors the
//! `blueprints/<name>/register_*` seam (MEM018/MEM106/MEM109, D049).

use bevy::prelude::*;

pub(in crate::windowed) mod agumon;
pub(in crate::windowed) mod renamon;

/// Register every Digimon's presentation into the engine registries. Called
/// once from `UiPlugin::build` after the engine inits the shared resources.
pub(super) fn register_all(app: &mut App) {
    agumon::register(app);
    renamon::register(app);
}
