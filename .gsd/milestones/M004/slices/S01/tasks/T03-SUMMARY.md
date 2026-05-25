---
id: T03
parent: S01
milestone: M004
key_files:
  - assets/digimon/agumon/vfx.ron
  - tests/animation/vfx_asset_load.rs
  - tests/animation.rs
key_decisions:
  - Modeled the scale curve as the ease-out outward fraction 1-(1-t)^2 sampled at 3 keyframes (0.0->0.0, 0.5->0.75, 1.0->1.0) rather than re-deriving the easing in data; the piecewise-linear evaluator reproduces the sampled values exactly at keyframes
  - Authored the central impact flash as a sibling effect (baby_flame.impact_flash) instead of on_expire chaining, keeping the tracer bullet minimal per the plan
  - Load test uses exact f32 equality at curve endpoints (authored keyframe values) but a 1e-6 epsilon for interpolated midpoints, since f32 lerp is deterministic but not bit-identical to decimal literals
duration: 
verification_result: passed
completed_at: 2026-05-25T10:06:42.744Z
blocker_discovered: false
---

# T03: Authored assets/digimon/agumon/vfx.ron (Baby Flame impact fan-out + central flash) and a headless load test asserting deterministic curve evaluation

**Authored assets/digimon/agumon/vfx.ron (Baby Flame impact fan-out + central flash) and a headless load test asserting deterministic curve evaluation**

## What Happened

Created the real authored Agumon VFX asset and proved the content-as-data path end-to-end with a headless load test.

The asset (`assets/digimon/agumon/vfx.ron`) is a typed `VfxAsset` map with two effects:
- `baby_flame.impact` (placement verb `impact.fan_out`): reproduces today's hardcoded shard-burst constants from src/windowed/render.rs — count 8 (BABY_FLAME_IMPACT_SHARD_COUNT), spread_px 64.0 (BABY_FLAME_IMPACT_SHARD_SPREAD_PX), ttl_ticks 5 (BABY_FLAME_IMPACT_SHARD_TTL). Its scale curve samples the existing ease-out outward fraction `1-(1-t)^2` from baby_flame_shard_offset at keyframes (0.0->0.0, 0.5->0.75, 1.0->1.0); its color curve holds the shard hue srgba(1.0,0.55,0.2,_) constant while the alpha linear-fades 0.9->0.0, matching baby_flame_shard_alpha.
- `baby_flame.impact_flash` (sibling, placement verb `impact.flash`): the bright central flash — count 1, ttl 2, color srgba(1.0,0.82,0.45,_) fading 0.95->0.0. Chose a sibling effect over on_expire chaining to keep the tracer bullet minimal (the plan permits either).

The load test (`tests/animation/vfx_asset_load.rs`, registered in tests/animation.rs) parses the file via compile-time `include_str!` matching anim_validation.rs (no .gsd/.planning path), asserts the impact effect resolves and its spawn_plan equals ImpactSpawnPlan { count: 8, spread_px: 64.0, ttl_ticks: 5 }, the flash sibling is present, a missing id resolves to None, and eval_scale/eval_color return the expected deterministic values at progress 0.0/0.5/1.0. Endpoint samples use exact f32 equality (authored keyframe values); interpolated midpoints use a 1e-6 epsilon since f32 lerp is deterministic but not bit-identical to a decimal literal.

Q4 / R012 re-verified: this presentation asset carries numeric appearance values but is a SEPARATE asset from the SpawnParticle command — no numeric gameplay payload was added to any Command/SpawnParticle surface.

## Verification

Ran the task verification command `cargo test --test animation vfx_asset_load`: 4 new tests pass (parse+presence, scale eased-spread, impact color fade, flash color fade), 0 failed. Ran the full `cargo test --test animation` harness to confirm no regression from the tests/animation.rs registry edit: 83 passed (79 prior + 4 new), 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation vfx_asset_load` | 0 | pass | 1000ms |
| 2 | `cargo test --test animation` | 0 | pass | 1000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation.rs`
