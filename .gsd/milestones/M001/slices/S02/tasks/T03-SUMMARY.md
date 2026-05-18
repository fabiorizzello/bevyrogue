---
id: T03
parent: S02
milestone: M001
key_files:
  - src/animation/plugin.rs
  - tests/clip_asset.rs
key_decisions:
  - Mirror graph-load readiness semantics for typed clip assets: observe load/modify events per handle and require successful `Assets<Clip>` reads before flipping `AnimationClipLoadState.ready`.
duration: 
verification_result: passed
completed_at: 2026-05-18T20:59:39.074Z
blocker_discovered: false
---

# T03: Registered typed Clip RON assets in AnimationAssetPlugin, added clip readiness resources/systems, and proved Agumon clip.ron loads through Bevy before ready flips.

**Registered typed Clip RON assets in AnimationAssetPlugin, added clip readiness resources/systems, and proved Agumon clip.ron loads through Bevy before ready flips.**

## What Happened

Updated `src/animation/plugin.rs` so `AnimationAssetPlugin` now registers both `RonAssetPlugin::<AnimGraph>` and `RonAssetPlugin::<Clip>`, initializes default clip paths for `digimon/agumon/clip.ron`, and exposes `AnimationClipPaths`, `AnimationClipHandles`, and `AnimationClipLoadState` beside the existing graph resources. Added startup clip loading plus update-time event tracking that mirrors the established graph readiness convention: log the load request, mark each handle loaded only after `LoadedWithDependencies` or `Modified`, and set `ready` only when every configured handle has emitted an event and is readable from `Assets<Clip>`. Added `tests/clip_asset.rs`, a headless `MinimalPlugins + AssetPlugin` smoke test that polls the real Bevy asset pipeline, proves clip readiness starts false, rejects premature ready before `Assets<Clip>` access works, and then inspects the loaded Agumon clip metadata and named frame ranges.

## Verification

Ran `cargo test --test clip_asset` to validate typed clip loading and readiness semantics through the real Bevy asset path, then ran `cargo test` to confirm the new clip wiring did not regress the existing graph loader or prior data/parity tests. Both commands passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test clip_asset` | 0 | ✅ pass | 4169ms |
| 2 | `cargo test` | 0 | ✅ pass | 50194ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/plugin.rs`
- `tests/clip_asset.rs`
