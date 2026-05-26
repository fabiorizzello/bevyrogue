---
id: T02
parent: S05
milestone: M006
key_files:
  - src/windowed/digimon/renamon/mod.rs
  - src/windowed/digimon/mod.rs
  - assets/digimon/renamon/stance.ron
  - assets/digimon/renamon/clip.ron
  - assets/digimon/renamon/anim_graph.ron
  - tests/windowed_only/renamon_extension_contract.rs
key_decisions:
  - Renamon extends `AnimationStancePaths` during `register(app)` so the stance asset is present before `AnimationAssetPlugin` Startup loading snapshots the path list.
  - Renamon leaves `diamond_storm_leaf` unmapped in the generic VFX registries so missing authored particle assets remain a no-op instead of introducing fake species-specific engine plumbing.
duration: 
verification_result: passed
completed_at: 2026-05-26T17:48:47.001Z
blocker_discovered: false
---

# T02: Added Renamon’s windowed presentation module, stance asset, and Diamond Storm release cue while keeping engine files species-agnostic.

**Added Renamon’s windowed presentation module, stance asset, and Diamond Storm release cue while keeping engine files species-agnostic.**

## What Happened

Created `src/windowed/digimon/renamon/mod.rs` as the Renamon-owned registration seam for windowed presentation data: it mutates `AnimationStancePaths` at app-build time so `digimon/renamon/stance.ron` is loaded before Startup asset loading, registers the Renamon sprite presentation for `UnitId(7)` with `renamon_stance` / `renamon_skill`, maps `diamond_storm` to `diamond_storm_cast`, and contributes a `renamon_ally` entry to the generic windowed demo registry. Updated `src/windowed/digimon/mod.rs` only at the aggregator seam to declare and call `renamon::register(app)`. Authored `assets/digimon/renamon/stance.ron` for idle/hurt/death/victory, extended `assets/digimon/renamon/clip.ron` with an `all` range covering frames 0-67, and added a `ReleaseKernel` cue on `diamond_storm_impact` in `assets/digimon/renamon/anim_graph.ron` so Diamond Storm releases the suspended timeline barrier. Replaced the existing `tests/windowed_only/renamon_extension_contract.rs` placeholder with a T02-specific source contract that locks the extension boundary, required assets, build-time stance-path mutation, and the no-fake-VFX expectation for `diamond_storm_leaf`.

## Verification

Verified the authored extension seam and asset contract with `cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture` (7 Renamon contract assertions passed). Verified the binary-side registration path with `cargo test --features windowed register_populates_the_windowed_registries -- --nocapture`, which ran `windowed::digimon::renamon::tests::register_populates_the_windowed_registries` successfully. Per project rule K001, no windowed binary/demo launch was attempted in auto mode.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only renamon_extension_contract -- --nocapture` | 0 | ✅ pass | 8083ms |
| 2 | `cargo test --features windowed register_populates_the_windowed_registries -- --nocapture` | 0 | ✅ pass | 5432ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/digimon/renamon/mod.rs`
- `src/windowed/digimon/mod.rs`
- `assets/digimon/renamon/stance.ron`
- `assets/digimon/renamon/clip.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/windowed_only/renamon_extension_contract.rs`
