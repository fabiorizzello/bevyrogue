---
estimated_steps: 8
estimated_files: 4
skills_used: []
---

# T02: Extend the VFX schema (typed params + anchor + size + texture), author all five effects, add load-time validation

Why: Removing VfxParticleKind in T03 removes the information the enum carried — per-kind ttl, size, anchor, texture, and the projectile→impact chain. That information must move into the typed, Reflect-able schema and the asset BEFORE the dispatcher can read it. This task is atomic: the serde-shape change to Placement/Appearance breaks the existing vfx.ron and the S01 schema/load tests simultaneously, so the asset migration and test updates land together to keep the build green.

Do:
1. In src/animation/vfx_asset.rs extend `Placement` to `{ verb: String, params: PlacementParams, anchor: PlacementAnchor }` (keep deny_unknown_fields) and extend `Appearance` with `size_px: f32` (per-particle quad size, replacing vfx_particle_size render.rs:886-895) and `texture: String` (a windowed-resolved image key, replacing the per-kind texture match render.rs:1087-1101). PlacementParams/PlacementAnchor come from T01.
2. Migrate the two existing effects in assets/digimon/agumon/vfx.ron (baby_flame.impact, baby_flame.impact_flash) to the new shape: impact placement (verb agumon/baby_flame/fan_out, params FanOut(spread_px:64.0), anchor TargetCenter, size_px 14.0, texture "baby_flame_impact"); impact_flash (verb agumon/baby_flame/static, params Static, anchor TargetCenter, size_px 26.0, texture "baby_flame_impact").
3. Author the three new effects reproducing the current hardcoded constants (so the K001 visual baseline is unchanged): baby_flame.charge (count 1, ttl_ticks 24, size_px 22.0, texture "baby_flame_charge", verb agumon/baby_flame/static + anchor Mouth; scale curve approximating the growth 0.42->~0.9 over life and color alpha 0.35->0.88 — drop the 4-tick micro-pulse as a documented K001 deviation), baby_flame.ember (count 7 = BABY_FLAME_EMBER_COUNT, ttl_ticks 24, size_px 11.0, texture "baby_flame_charge", verb agumon/baby_flame/converge_inward + params ConvergeInward(radius_px:58.0, omega:0.9), anchor Mouth; color alpha 0.9->0.0), baby_flame.projectile (count 1, ttl_ticks 4, size_px 16.0, texture "baby_flame_projectile", verb agumon/baby_flame/arc_launch + params ArcLaunch, anchor CasterCenter, color srgba ~1.0/0.45/0.15; **on_expire: "baby_flame.impact"** — this is the data replacement for the hardcoded projectile->impact burst at render.rs:1428-1436, and gives on_expire its first real chained use).
4. Add a pure headless load-validation fn in vfx_asset.rs, e.g. `validate_effects(asset: &VfxAsset, known_verbs: &[&str]) -> Result<(), VfxValidationError>` (or returns the first offending (effect_id, reason)) that checks every effect.placement.verb is in known_verbs and every on_expire id resolves to a present effect. This is the CONTEXT-mandated load-time validation error path (which Digimon/effect/verb), surfaced as data — no panic.
5. Update tests/animation/vfx_asset_schema.rs and tests/animation/vfx_asset_load.rs for the new shape; add tests: round-trip of all 5 effects; deny_unknown_fields still rejects an extra field; Reflect introspection lists the new Placement.params/anchor and Appearance.size_px/texture fields (D034); load asserts presence of all 5 effect ids, spawn_plan equality for charge/ember/projectile, on_expire of baby_flame.projectile == baby_flame.impact, and sampled eval_scale/eval_color at progress 0.0/0.5/1.0; validate_effects returns Ok for the real asset and Err naming the verb for a synthetic asset with an unregistered verb id and for a dangling on_expire id (negative tests, Q7).

Done when: cargo test --test animation passes (updated schema/load tests + new validation tests) and cargo build (headless) is clean. The windowed glue from S01 still compiles because it only reads via resolve_effect/spawn_plan/eval_* (added fields are additive to its view) — confirm in T03's build.

## Inputs

- `src/animation/vfx_asset.rs`
- `assets/digimon/agumon/vfx.ron`
- `src/windowed/render.rs`
- `tests/animation/vfx_asset_schema.rs`
- `tests/animation/vfx_asset_load.rs`

## Expected Output

- `src/animation/vfx_asset.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_schema.rs`
- `tests/animation/vfx_asset_load.rs`

## Verification

cargo test --test animation

## Observability Impact

Adds validate_effects as the headless load-time validation surface; its Err payload names the first invalid (effect, verb/on_expire) pair. Negative tests assert the error path fires for unregistered verb id and dangling on_expire.
