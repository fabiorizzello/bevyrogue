---
id: T02
parent: S01
milestone: M004
key_files:
  - src/animation/vfx_asset.rs
  - tests/animation/vfx_asset_eval.rs
  - tests/animation.rs
key_decisions:
  - Shared generic eval_curve helper sorts keyframe indices (not keyframes) by f32::total_cmp so authored order is preserved and the total order is deterministic (R004)
  - Empty-curve defaults: 1.0 scale and opaque-white [1,1,1,1] color, exposed as DEFAULT_SCALE/DEFAULT_COLOR consts
  - ImpactSpawnPlan derives PartialEq (not Eq) because spread_px is f32; spawn_plan is a plain reader of Appearance
  - resolve_effect returns Option (None for absent ids) to support the windowed log-and-fallback path rather than panicking
duration: 
verification_result: passed
completed_at: 2026-05-25T10:04:28.270Z
blocker_discovered: false
---

# T02: Added pure deterministic appearance-curve evaluator (eval_scale/eval_color), effect resolver, and ImpactSpawnPlan constructor with 11 passing eval tests

**Added pure deterministic appearance-curve evaluator (eval_scale/eval_color), effect resolver, and ImpactSpawnPlan constructor with 11 passing eval tests**

## What Happened

Implemented R004-pure, render-free appearance math in src/animation/vfx_asset.rs so the windowed render path (T04) stays thin glue. Added: `eval_scale(&ScaleCurve, f32) -> f32` and `eval_color(&ColorCurve, f32) -> [f32; 4]`, both delegating to a shared generic `eval_curve` helper that performs piecewise-linear interpolation. The helper sorts keyframe *indices* (not the keyframes themselves, preserving authored order) by `t` via `f32::total_cmp` for a deterministic total order, clamps `progress` to [0,1], returns the first value before the first keyframe, the last value after the last, a documented default for empty curves (1.0 scale / opaque white [1,1,1,1]), and the linear interpolant within the bracketing segment (guarding division by a zero span from duplicate `t`s). Added `resolve_effect<'a>(&'a VfxAsset, &str) -> Option<&'a EffectDef>` (returns None for absent ids so the windowed layer can log + fall back per slice verification) and a `Copy` `ImpactSpawnPlan { count, spread_px, ttl_ticks }` with `spawn_plan(&EffectDef) -> ImpactSpawnPlan` reading from Appearance. All functions are I/O-free and use no Bevy world/render types. Wrote tests/animation/vfx_asset_eval.rs (11 tests) and registered it in tests/animation.rs. Skipped the code-optimizer multi-agent audit: it targets existing-code perf anti-patterns via pattern search, not the authoring of small new pure interpolation functions; wrote allocation-light idiomatic code directly instead.

## Verification

Ran `cargo test --test animation vfx_asset_eval` — 11 passed, 0 failed, 0 ignored (exit 0, ~5.1s). Tests cover the Q7 boundary/negative matrix: endpoint equality at progress 0.0 and 1.0, midpoint linear interpolant (scale 0.6; color per-channel [1.0,0.5,0.1,0.5]), clamping below 0 and above 1, empty-curve documented defaults, single-keyframe constancy across all progress, determinism (1000 repeated calls bit-identical), and resolve_effect None/Some + spawn_plan reading Appearance. Also ran `cargo clippy --lib` and `cargo clippy --test animation` (exit 0); no warnings reference vfx_asset — remaining warnings are pre-existing in other modules.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation vfx_asset_eval` | 0 | pass | 5120ms |
| 2 | `cargo clippy --test animation` | 0 | pass (no vfx_asset warnings) | 5097ms |
| 3 | `cargo clippy --lib` | 0 | pass (no vfx_asset warnings) | 5097ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/animation/vfx_asset.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation.rs`
