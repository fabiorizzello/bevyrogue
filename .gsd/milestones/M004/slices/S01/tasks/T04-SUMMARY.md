---
id: T04
parent: S01
milestone: M004
key_files:
  - src/windowed/render.rs
  - tests/windowed_only/vfx_asset_impact_render.rs
  - tests/windowed_only.rs
key_decisions:
  - Resolve the impact EffectDef once per frame in advance_vfx_particles into Option<&EffectDef>; Some = data path, None = hardcoded fallback. Keeps the hot loop borrow-clean and makes the fallback a single branch.
  - Added an Option<u32> ttl_override to spawn_vfx_particle rather than keying ttl off VfxParticleKind, so data-driven effects source ttl_ticks from spawn_plan; existing kinds pass None and keep their per-kind defaults.
  - Used the authored effect id 'baby_flame.impact' (the dotted id in vfx.ron / T03) rather than the plan's literal 'baby_flame_impact' (which is the particle name, not the effect id) — confirmed against assets/digimon/agumon/vfx.ron and the T03 load test.
  - Kept the hardcoded BABY_FLAME_IMPACT_SHARD_* constants and baby_flame_shard_offset/alpha as the fallback path instead of deleting them, since the slice contract requires visible fall-back on missing/malformed asset.
  - Test exercises the headless lib contract (resolve_effect/spawn_plan/eval_*) under the windowed feature because src/windowed is binary-private and cannot be reached from integration tests.
duration: 
verification_result: passed
completed_at: 2026-05-25T10:14:17.677Z
blocker_discovered: false
---

# T04: Ported the Baby Flame impact fan-out to render from the owned vfx.ron data path (windowed glue) with a hardcoded fallback

**Ported the Baby Flame impact fan-out to render from the owned vfx.ron data path (windowed glue) with a hardcoded fallback**

## What Happened

Wired the windowed render layer to source the Baby Flame impact fan-out from the owned `assets/digimon/agumon/vfx.ron` instead of hardcoded constants, proving the schema reaches pixels (first hardcoded effect ported to data).

Render-side (src/windowed/render.rs, binary crate):
- Registered `RonAssetPlugin::<VfxAsset>::new(&["ron"])` in `RenderPlugin::build`, windowed-gated, mirroring the AnimGraph/Clip loaders in src/animation/plugin.rs. Multiple `.ron` RonAssetPlugins for distinct asset types coexist (same pattern as src/data/mod.rs).
- Added an `AgumonVfx { handle: Handle<VfxAsset> }` resource loaded at Startup via `load_agumon_vfx` from `digimon/agumon/vfx.ron`.
- `advance_vfx_particles` now resolves the impact effect once per frame: `resolve_effect(asset, "baby_flame.impact")` → `Option<&EffectDef>`. `Some` drives the data path; `None` (asset not loaded, load failed, or effect id absent) falls back to the prior hardcoded path.
- `spawn_baby_flame_impact_burst` sources shard count + lifetime from `spawn_plan(effect)` when data is live (fallback: `BABY_FLAME_IMPACT_SHARD_COUNT`). The per-tick impact-shard branch sources outward distance from `spread_px * eval_scale(scale_curve, progress)` and full rgba from `eval_color(color_curve, progress)`, replacing `baby_flame_shard_offset`/`baby_flame_shard_alpha` for THIS effect only.
- Added an `Option<u32>` ttl_override parameter to `spawn_vfx_particle` so data-driven effects override the per-kind default ttl; all four existing call sites (charge core, projectile, embers, detonate) pass `None`.
- VfxParticleKind, vfx_particle_kind, and the charge/ember/projectile/burner kinds are untouched (S02 removes the enum).

Failure visibility (Q5): `diagnose_agumon_vfx_load` warns once on target "windowed.agumon_playback" with effect id + path + reason when `LoadState::Failed` (missing/malformed vfx.ron); a loaded-but-missing effect id warns once inside `advance_vfx_particles`. Both fall back to the hardcoded impact path — no panic, VFX still renders.

Tests: added tests/windowed_only/vfx_asset_impact_render.rs (feature-gated `#![cfg(feature = "windowed")]`, registered in tests/windowed_only.rs). Because src/windowed is binary-private, the test pins the lib contract render.rs consumes: the authored RON resolves to baby_flame.impact, `spawn_plan` yields count=8/spread=64/ttl=5, and eval_scale/eval_color reproduce the per-tick outward distance (spread_px * frac) and rgba the shard branch writes (e.g. 0.75*64=48px at midpoint), plus the missing-id → None fallback.

Requirement re-verification (Q4): R012 — no numeric gameplay payload added to any serialized command; vfx.ron is a separate presentation asset (unchanged command surface). R016 — resolver/eval live in the headless lib (src/animation/vfx_asset.rs); only RonAssetPlugin registration + handle live windowed-side; `cargo build` (headless) stays green, confirming no windowed dep leak.

## Verification

cargo build --features windowed compiles clean (exit 0). The new windowed_only test passes: `cargo test --features windowed --test windowed_only vfx_asset_impact` → 3 passed, 0 failed. Headless `cargo build` is green (R016: no windowed dep leak into the headless lib). Full headless `cargo test` is green across all scope harnesses (0 failures). Visual confirmation of the fan-out via `cargo winx` is deferred to human UAT per K001 (auto-mode must not open a window).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 7900ms |
| 2 | `cargo test --features windowed --test windowed_only vfx_asset_impact` | 0 | pass | 8720ms |
| 3 | `cargo build` | 0 | pass | 13180ms |
| 4 | `cargo test` | 0 | pass | 0ms |

## Deviations

Plan referenced effect id "baby_flame_impact"; used the actual authored id "baby_flame.impact" (the former is the particle name). No contract impact — the resolver key is the only consumer.

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only.rs`
