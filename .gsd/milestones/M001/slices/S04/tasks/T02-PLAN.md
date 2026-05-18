---
estimated_steps: 8
estimated_files: 1
skills_used: []
---

# T02: Implement Catalog Sync and Dynamic Discovery

Implement dynamic discovery and catalog synchronization in the animation plugin to move beyond Agumon-only hardcoding (R003, R007).

Steps:
1. Expand `DEFAULT_ANIM_GRAPH_PATHS` and `DEFAULT_ANIM_CLIP_PATHS` in `src/animation/plugin.rs` to include the full roster (Agumon, Gabumon, Renamon, etc.).
2. Implement `sync_validation_catalogs` system in `src/animation/plugin.rs` that populates `AnimationValidationCatalogs` from `SkillBookHandle` and `StatusEffectKind` enum variants once `DataReady` is present.
3. Ensure `validate_animation_assets` triggers when catalogs or assets change.
4. Update `track_animation_graph_loads` and `track_animation_clip_loads` to be resilient to missing files in the roster (logging a warning instead of blocking readiness for the whole app).

Done when:
- `cargo test --test anim_asset_validation` passes and covers both Agumon and Renamon.

## Inputs

- `src/animation/plugin.rs`
- `src/data/mod.rs`
- `src/combat/mechanics/status_effect.rs`

## Expected Output

- `src/animation/plugin.rs`

## Verification

cargo test --test anim_asset_validation
