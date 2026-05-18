---
id: T02
parent: S01
milestone: M001
key_files:
  - src/animation/mod.rs
  - src/animation/plugin.rs
  - assets/digimon/agumon/anim_graph.ron
  - tests/anim_graph_asset.rs
key_decisions:
  - Made `AnimationGraphPaths` a data-driven resource backed by a default path list so future roster additions can extend asset coverage without new Rust enum branches or hard-coded loader branches.
  - Gated `AnimationGraphLoadState.ready` on both observed `AssetEvent::{LoadedWithDependencies|Modified}` and successful `Assets<AnimGraph>` lookup so the loader cannot report readiness before a typed asset is actually readable.
duration: 
verification_result: passed
completed_at: 2026-05-18T20:48:56.270Z
blocker_discovered: false
---

# T02: Added a typed AnimGraph asset plugin, real Agumon `anim_graph.ron`, and a headless loader test that proves readiness only flips after the asset is readable.

**Added a typed AnimGraph asset plugin, real Agumon `anim_graph.ron`, and a headless loader test that proves readiness only flips after the asset is readable.**

## What Happened

Added a new `AnimationAssetPlugin` under `src/animation/plugin.rs` and re-exported it from `src/animation/mod.rs`. The plugin registers `RonAssetPlugin::<AnimGraph>`, loads a data-driven list of graph asset paths via `AssetServer`, stores explicit `AnimationGraphHandles`, and maintains `AnimationGraphLoadState` by watching `MessageReader<AssetEvent<AnimGraph>>` in the same style as the existing data loader. The ready flag stays false until every configured graph both emits a load/modify event and is readable from `Assets<AnimGraph>`, with info logs for requested loads, per-asset load/modify events, and all-graphs-ready. Added the real `assets/digimon/agumon/anim_graph.ron` asset using the typed T01 schema, mapped to the atlas `skill` clip frame range, and modeled a minimal Baby Flame cast/impact/recover flow using canon identifiers like `baby_flame` and `Heated`. Added `tests/anim_graph_asset.rs`, a headless Bevy integration test that mounts `AssetPlugin` against the repo `assets/` directory, waits with a bounded timeout for the typed asset to become available, asserts the loader never flips ready before availability, and verifies the loaded graph’s clip, entry node, transitions, particle command, damage command, modifier, and target shape.

## Verification

Verified the new loader seam with fresh dedicated and broader Rust checks. `cargo test --test anim_graph_asset` passed after formatting, proving the headless app can load `assets/digimon/agumon/anim_graph.ron` through `AnimationAssetPlugin`, that `AnimationGraphLoadState` starts false, and that readiness only flips after a load/modify event and successful `Assets<AnimGraph>` lookup. A broader `cargo test --test anim_graph_parse --test anim_graph_asset` run passed to confirm the real asset loader still agrees with the closed typed schema and out-of-vocabulary rejection tests from T01. `cargo check` also passed in the same sweep, confirming the new plugin exports compile cleanly across the crate.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test anim_graph_asset` | 0 | ✅ pass | 426ms |
| 2 | `cargo test --test anim_graph_parse --test anim_graph_asset && cargo check` | 0 | ✅ pass | 2749ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/mod.rs`
- `src/animation/plugin.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
