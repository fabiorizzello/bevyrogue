---
estimated_steps: 15
estimated_files: 2
skills_used: []
---

# T03: Wire Clip into AnimationAssetPlugin and asset-load smoke test

Expected executor skills: bevy, rust-best-practices, rust-testing, verify-before-complete, observability.

Why: Direct RON parsing proves schema shape, but R003 specifically requires clip.ron to load as a typed Bevy asset. The existing S01 AnimationAssetPlugin is the right cohesive module boundary for this wiring.

Do:
1. Update src/animation/plugin.rs to register RonAssetPlugin::<Clip>::new(&["ron"]) in AnimationAssetPlugin.
2. Add default clip path configuration for digimon/agumon/clip.ron while keeping graph path configuration intact.
3. Add clip handle and load-state resources analogous to AnimationGraphHandles and AnimationGraphLoadState, e.g. AnimationClipPaths, AnimationClipHandles, and AnimationClipLoadState.
4. Add startup loading and update tracking systems for Clip assets. The ready flag must flip only after load/modify events have been observed and every configured handle is readable from Assets<Clip>, matching the MEM038/MEM037 readiness convention.
5. Add tests/clip_asset.rs with a MinimalPlugins + AssetPlugin headless app that loads AnimationAssetPlugin, polls updates, asserts clip readiness starts false, asserts readiness does not precede asset availability, and then inspects the loaded Clip geometry and ranges.
6. Run the focused clip asset test and then the broad cargo test suite to catch regression in S01 graph loading and existing data tests.

Failure modes:
- Missing asset file: clip load state never reaches ready and the test times out with loaded flags.
- Malformed RON: Assets<Clip> remains unreadable and the test localizes the failing asset path via readiness assertions/logs.
- Premature ready flag: test fails before ready can mask an unreadable typed asset.

Load profile: trivial; one default clip asset in S02. Future roster expansion should reuse the vector path configuration without changing schema.

Done when: cargo test --test clip_asset and cargo test pass, and S03 can import Clip plus inspect configured Clip asset readiness through the animation module.

## Inputs

- `src/animation/plugin.rs`
- `src/animation/clip.rs`
- `assets/digimon/agumon/clip.ron`
- `tests/anim_graph_asset.rs`

## Expected Output

- `src/animation/plugin.rs`
- `tests/clip_asset.rs`

## Verification

cargo test --test clip_asset

## Observability Impact

Adds clip asset load request, per-asset load/modify, and ready-state diagnostics so future failures can distinguish missing events, unreadable typed assets, and successfully loaded geometry.
