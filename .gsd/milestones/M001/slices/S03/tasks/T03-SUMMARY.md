---
id: T03
parent: S03
milestone: M001
key_files:
  - src/animation/plugin.rs
  - src/animation/validation.rs
  - src/animation/validation/types.rs
  - src/animation/validation/graph.rs
  - src/animation/validation/predicate.rs
  - src/animation/validation/command.rs
  - tests/anim_asset_validation.rs
  - assets/test/animation_validation/valid_anim_graph.ron
  - assets/test/animation_validation/valid_clip.ron
  - assets/test/animation_validation/broken_anim_graph.ron
  - assets/test/animation_validation/broken_clip.ron
key_decisions:
  - Injected `AnimationValidationCatalogs` and `AnimationValidationState` as explicit resources so plugin boot validation stays inside `src/animation` without importing project-data internals.
  - Matched graphs to clips by the graph `clip` range id across loaded clip assets, then emitted typed diagnostics when no clip asset exposed the required range.
  - Split the validator into scoped submodules to satisfy the repository LOC architectural guard while preserving the existing `validate_anim_graph` API.
duration: 
verification_result: passed
completed_at: 2026-05-18T21:26:02.886Z
blocker_discovered: false
---

# T03: Added plugin-level headless animation asset validation state with typed pass/fail diagnostics plus fixture-backed Bevy integration tests.

**Added plugin-level headless animation asset validation state with typed pass/fail diagnostics plus fixture-backed Bevy integration tests.**

## What Happened

Extended `src/animation/plugin.rs` so `AnimationAssetPlugin` now initializes `AnimationValidationCatalogs` and `AnimationValidationState`, watches graph/clip asset events, and runs headless validation once configured `AnimGraph` and `Clip` assets are both readable. The validator stays adapter-driven: catalogs are injected as a resource, and graph-to-clip pairing is resolved generically by matching the graph `clip` id against loaded clip ranges instead of importing project data into `src/animation`. Added `tests/anim_asset_validation.rs` plus committed valid/broken fixtures under `assets/test/animation_validation/` to prove plugin-level pass/fail behavior in a minimal Bevy app. During regression, `cargo test` surfaced the repository LOC guard on `src/animation/validation.rs`, so I split that module into scoped submodules (`validation/types.rs`, `graph.rs`, `predicate.rs`, `command.rs`) without changing the public validation API.

## Verification

Verified the new plugin surface with `cargo test --test anim_asset_validation`, which now passes for both valid and broken fixture sets and confirms `AnimationValidationState` reaches `Ready` for valid assets and `Failed` with typed diagnostics for broken assets. Re-ran full `cargo test` after the validation-module split to confirm existing `anim_graph_asset` and `clip_asset` readiness coverage stayed green and the source-file LOC guard passed again.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test anim_asset_validation` | 0 | ✅ pass | 5986ms |
| 2 | `cargo test` | 0 | ✅ pass | 38084ms |

## Deviations

Split `src/animation/validation.rs` into scoped submodules after full regression exposed the source-file LOC cap; this was not explicitly listed in the task plan but was required to keep `cargo test` green.

## Known Issues

None.

## Files Created/Modified

- `src/animation/plugin.rs`
- `src/animation/validation.rs`
- `src/animation/validation/types.rs`
- `src/animation/validation/graph.rs`
- `src/animation/validation/predicate.rs`
- `src/animation/validation/command.rs`
- `tests/anim_asset_validation.rs`
- `assets/test/animation_validation/valid_anim_graph.ron`
- `assets/test/animation_validation/valid_clip.ron`
- `assets/test/animation_validation/broken_anim_graph.ron`
- `assets/test/animation_validation/broken_clip.ron`
