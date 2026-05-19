# M001: Animation asset pipeline foundation — Research (S04)

## Summary

Slice S04 moves the animation pipeline from an Agumon-only proof into a roster-ready operational state. This involves authoring animation assets for the remaining Digimon (Gabumon, Renamon, etc.), ensuring the `AnimationAssetPlugin` dynamically loads the full roster, and verifying the hot-reload authoring loop in the `windowed` environment. 

The primary technical challenge is ensuring that the `AnimationValidationCatalogs` are correctly synchronized with real project data (SkillBooks, StatusEffects, and Particles) so that the generic validator established in S03 can prove the integrity of the entire roster without hardcoding. We will leverage the existing `DataReady` signal from `DataPlugin` to trigger catalog hydration before the first validation pass.

## Recommendation

We should adopt a "Roster Discovery" pattern in `AnimationAssetPlugin` that mirrors `DataPlugin`, automatically finding animation assets based on the established directory structure (`assets/digimon/*/`). We will author `clip.ron` and `anim_graph.ron` for Renamon as the primary "non-Agumon" proof, as her atlas and skill data are already well-defined in the repository.

For hot-reload, we will rely on Bevy's `watch_for_changes_override` (already enabled in `src/windowed.rs`). The `AnimationAssetPlugin` is already wired to re-validate on `AssetEvent::Modified`, so the proof will focus on ensuring the `AnimationValidationState` correctly reflects live edits and that the app remains stable even when invalid RON is temporarily saved.

## Implementation Landscape

### Key Files

- `src/animation/plugin.rs` — Needs to move from static `DEFAULT_ANIM_GRAPH_PATHS` to dynamic discovery or an expanded roster list. Add a `sync_validation_catalogs` system.
- `src/animation/validation/types.rs` — No changes likely needed, but ensures `AnimationValidationCatalogs` remains the source of truth.
- `assets/digimon/renamon/clip.ron` — **New file.** Authored from `renamon_atlas.json`.
- `assets/digimon/renamon/anim_graph.ron` — **New file.** Authored from `renamon/skills.ron`.
- `src/windowed.rs` — Add a visual "Validation Status" indicator to the Roster panel to facilitate the hot-reload UAT.

### Build Order

1.  **Roster Expansion**: Author Renamon's `clip.ron` and `anim_graph.ron`. This proves R007 immediately.
2.  **Catalog Synchronization**: Implement a system in `AnimationAssetPlugin` that populates `AnimationValidationCatalogs` from the global `SkillBook` and `StatusEffectKind` enum. This unblocks accurate validation for the whole roster.
3.  **Dynamic Loading**: Update `AnimationAssetPlugin` to load all detected animation assets.
4.  **Hot-Reload UAT**: Run `cargo run --features windowed`, edit a RON file, and verify logs/UI reflect the change and validation status.

### Validation Catalogs Hydration
The `AnimationValidationCatalogs` should be populated as follows:
- **Statuses**: Iterate over all variants of `StatusEffectKind`.
- **Skills**: Extract all `SkillId`s from the aggregated `SkillBook`.
- **Params**: Derived from the combat system's supported parameter keys.
- **Particles**: Currently string-based; should be populated from a known list of VFX names used in the project (e.g., "baby_flame", "diamond_storm").

### Hot Reload UAT Procedure
1.  Launch app: `cargo run --features windowed`.
2.  Observe "Animation Validation Ready" log.
3.  Edit `assets/digimon/agumon/anim_graph.ron` to introduce a typo in a `NodeId`.
4.  Observe "Animation Validation Failed" warning with a typed diagnostic (e.g., `UnknownNodeReference`).
5.  Fix the typo.
6.  Observe "Animation Validation Ready" log returning.
7.  Verify the app did not panic or lose world state.
