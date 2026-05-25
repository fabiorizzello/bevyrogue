---
id: T02
parent: S02
milestone: M004
key_files:
  - src/animation/vfx_asset.rs
  - assets/digimon/agumon/vfx.ron
  - tests/animation/vfx_asset_schema.rs
  - tests/animation/vfx_asset_load.rs
  - tests/animation/vfx_asset_eval.rs
key_decisions:
  - Placement gains typed params + anchor and Appearance gains size_px + texture, all additive to the windowed read-path so S01 glue still compiles; T03 deletes VfxParticleKind and consumes them.
  - validate_effects returns the first offending (effect_id, verb|missing) pair as data (Result/Err enum, deterministic BTreeMap order) rather than panicking — the headless load-time validation surface.
  - Charge's 4-tick micro-pulse is dropped as a documented K001 visual deviation; growth (0.42->0.9, maxing at age 6 = 0.25 life) and alpha (0.35->0.88) are encoded as keyframe curves over normalized progress.
  - Appearance.spread_px is retained alongside FanOut.spread_px param because render.rs's existing shard path reads spawn_plan().spread_px; the duplication is intentional for T03 compatibility.
  - Placement verb ids adopt the namespaced agumon/baby_flame/<verb> convention; known_verbs is supplied by the caller (tests) since Registry registration is windowed/T03.
duration: 
verification_result: passed
completed_at: 2026-05-25T10:38:20.718Z
blocker_discovered: false
---

# T02: Extended the VFX schema with typed placement params/anchor + per-particle size/texture, authored all five Baby Flame effects (incl. projectile->impact on_expire chain), and added headless validate_effects load-time validation.

**Extended the VFX schema with typed placement params/anchor + per-particle size/texture, authored all five Baby Flame effects (incl. projectile->impact on_expire chain), and added headless validate_effects load-time validation.**

## What Happened

Extended `Placement` in src/animation/vfx_asset.rs from `{ verb }` to `{ verb, params: PlacementParams, anchor: PlacementAnchor }` (params/anchor reused from T01), and added `size_px: f32` + `texture: String` to `Appearance` — moving the per-kind ttl/size/anchor/texture and the projectile→impact chain off the soon-to-be-deleted VfxParticleKind enum and into the typed, Reflect-able, deny_unknown_fields schema BEFORE T03's dispatcher reads it.

Migrated assets/digimon/agumon/vfx.ron to the new shape and authored all five effects, reproducing the K001 visual baseline from render.rs constants: charge (count 1, ttl 24, size 22, static@Mouth, scale 0.42->0.9 over first quarter, alpha 0.35->0.88 — 4-tick micro-pulse dropped as a documented K001 deviation), ember (count 7=BABY_FLAME_EMBER_COUNT, ttl 24, size 11, converge_inward radius 58/omega 0.9 @Mouth, alpha 0.9->0.0), projectile (count 1, ttl 4, size 16, arc_launch@CasterCenter, on_expire: baby_flame.impact — the data replacement for the hardcoded projectile->impact burst), impact (count 8, spread 64, ttl 5, size 14, fan_out@TargetCenter), and impact_flash (count 1, ttl 2, size 26, static@TargetCenter). Verb ids use the namespaced `agumon/baby_flame/<verb>` convention.

Added the pure headless `validate_effects(asset, known_verbs) -> Result<(), VfxValidationError>` with a two-variant error enum (UnknownVerb / DanglingOnExpire) that returns the first offending (effect_id, verb|missing) pair in deterministic BTreeMap order as data — never panics — implementing the CONTEXT-mandated load-time validation surface.

Updated all three animation test files for the new serde shape (vfx_asset_schema.rs, vfx_asset_load.rs, vfx_asset_eval.rs — the third was an unlisted-but-required compile fix) and added tests: 5-effect round-trip, deny_unknown_fields on the new Placement field, Reflect introspection of Placement.{verb,params,anchor} and Appearance.{...,size_px,texture}, presence of all 5 ids, spawn_plan equality for charge/ember/projectile, projectile.on_expire == baby_flame.impact, sampled eval_scale/eval_color, validate_effects Ok for the real asset, and Err naming the verb for an unregistered verb id and naming the target for a dangling on_expire (Q7 negatives).

render.rs was reference-only this task (windowed, #[cfg]-gated, not compiled headless); T03 deletes VfxParticleKind and ports the dispatcher to read these fields.

## Verification

cargo build (headless) clean, exit 0. cargo test --test animation: 100 passed, 0 failed, exit 0 — includes the updated schema/load/eval tests and the new round-trip, Reflect, on_expire-chain, and validate_effects positive+negative (unregistered verb, dangling on_expire) tests. The slice-level load-validation requirement is met by validate_effects + its two negative tests; the asset's K001 numeric baseline is asserted via spawn_plan/eval_* sampling.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build` | 0 | pass | 5720ms |
| 2 | `cargo test --test animation` | 0 | pass (100 passed; 0 failed) | 190ms |

## Deviations

Updated tests/animation/vfx_asset_eval.rs (not in the plan's Expected Output) because it constructs Placement/Appearance literals and inline RON directly — required to keep `cargo test --test animation` compiling after the serde shape change.

## Known Issues

none

## Files Created/Modified

- `src/animation/vfx_asset.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_schema.rs`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
