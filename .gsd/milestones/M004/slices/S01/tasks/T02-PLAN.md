---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T02: Implement pure deterministic appearance curve evaluator + effect-appearance resolver

Why: R004 requires all appearance verb math to be pure and headless-testable; keeping the curve eval and asset->spawn-params resolution in the headless lib lets the windowed render path (T04) be thin glue rather than untested rendering logic. Do: in src/animation/vfx_asset.rs add pure functions: `eval_scale(curve: &ScaleCurve, progress: f32) -> f32` and `eval_color(curve: &ColorCurve, progress: f32) -> [f32; 4]`, each performing piecewise-linear interpolation between keyframes sorted by `t`, clamping `progress` to [0,1], returning the first keyframe value for progress before the first `t`, the last for progress after the last `t`, and a sensible default (1.0 scale / opaque white) for an empty curve. Add `resolve_effect<'a>(asset: &'a VfxAsset, effect_id: &str) -> Option<&'a EffectDef>` and a small pure `ImpactSpawnPlan { count: u32, spread_px: f32, ttl_ticks: u32 }` constructor `spawn_plan(effect: &EffectDef) -> ImpactSpawnPlan` reading from Appearance, so the windowed layer asks the lib for concrete spawn parameters. All functions are Bevy-render-free (may use bevy math/Reflect already in the lib). Write tests/animation/vfx_asset_eval.rs and register it in tests/animation.rs covering (Q7 negative/boundary): eval at progress 0.0 and 1.0 equals the endpoint keyframes; eval at a midpoint equals the linear interpolant; progress clamps below 0 and above 1; empty curve returns the documented default; single-keyframe curve returns that keyframe for all progress; eval is deterministic (same input -> identical output across repeated calls). Done when: vfx_asset_eval tests pass and the functions are pure (no I/O, no Bevy world/render types). Requirement impact (Q4): exercises validated R004 determinism; no contract broken.

## Inputs

- `src/animation/vfx_asset.rs`
- `tests/animation.rs`

## Expected Output

- `src/animation/vfx_asset.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation vfx_asset_eval
