---
id: S01
parent: M004
milestone: M004
provides:
  - Typed VfxAsset schema in headless lib (Serialize+Deserialize+Reflect, deny_unknown_fields)
  - Pure deterministic eval_scale/eval_color/resolve_effect/spawn_plan API
  - assets/digimon/agumon/vfx.ron with baby_flame.impact and baby_flame.impact_flash
  - Windowed data path for Baby Flame impact fan-out with hardcoded fallback
  - RonAssetPlugin::<VfxAsset> + AgumonVfx handle resource pattern for S02/S03 to extend
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - VfxAsset omits explicit TypePath derive — Reflect's proc-macro already emits it, causing E0119 if both are listed; Asset only needs TypePath to exist (MEM070)
  - EffectId is a transparent String newtype with Eq+Ord+Hash serving as BTreeMap key and on_expire chain reference
  - Empty-curve defaults are documented constants: DEFAULT_SCALE=1.0 / DEFAULT_COLOR=[1,1,1,1] (opaque white)
  - eval_curve sorts keyframe indices by f32::total_cmp to preserve authored order while ensuring a deterministic total order
  - Baby Flame impact_flash modeled as a sibling effect rather than on_expire chain to keep the tracer bullet minimal
  - ttl_override: Option<u32> added to spawn_vfx_particle so data-driven effects override per-kind defaults without touching existing call sites
  - Effect id in vfx.ron is 'baby_flame.impact' (dotted namespace); plan's 'baby_flame_impact' was the particle constant name, not the authored id
patterns_established:
  - Data-driven windowed rendering with headless fallback via RonAssetPlugin + Handle resource + Some/None branch per frame (MEM072)
  - Piecewise-linear curve eval with sorted-indices pattern for deterministic authored-order-safe keyframe interpolation (MEM073)
  - Compile-time include_str! for headless RON asset load tests — mirrors anim_validation.rs precedent, no file I/O at test runtime
observability_surfaces:
  - diagnose_agumon_vfx_load: warns once on target 'windowed.agumon_playback' with effect id + path + reason on LoadState::Failed
  - advance_vfx_particles: warns once on target 'windowed.agumon_playback' with effect id when resolve_effect returns None (loaded asset missing the requested id)
drill_down_paths:
  - S02 depends on src/animation/vfx_asset.rs resolver/eval API — extend placement verbs in Registry<E> and remove VfxParticleKind/vfx_particle_kind
  - S03 depends on vfx.ron data path being proven (this slice) — add variant selection + Baby Burner detonate port
duration: ""
verification_result: passed
completed_at: 2026-05-25T10:17:12.332Z
blocker_discovered: false
---

# S01: Owned vfx.ron schema + appearance curve eval (tracer bullet)

**Established the editor-ready typed VfxAsset schema, pure deterministic curve evaluator, and authored vfx.ron; Baby Flame impact fan-out now renders from the data path with a hardcoded fallback.**

## What Happened

Four tasks delivered the tracer bullet end-to-end.

**T01 — VfxAsset schema:** Created `src/animation/vfx_asset.rs` defining `VfxAsset { effects: BTreeMap<EffectId, EffectDef> }` deriving `Asset, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect` with `#[serde(deny_unknown_fields)]`. `EffectDef` carries a typed `Placement { verb: String }` (Registry id by string; resolution deferred to S02), `Appearance { count, spread_px, ttl_ticks, scale: ScaleCurve, color: ColorCurve }`, and `on_expire: Option<EffectId>`. Every field type derives the full set including `Reflect` for D034 editor-readiness. Notable deviation: `TypePath` was dropped from the VfxAsset derive list because `Reflect`'s proc-macro already emits a TypePath impl (E0119 conflict); `Asset` only needs TypePath to exist (captured MEM070). No Cargo.toml change needed — `bevy_reflect` is a hard dep of `bevy_internal` and already resolves under `default-features=false`. Three schema tests pass: RON round-trip, deny_unknown_fields rejection, and `<Appearance as bevy::reflect::Struct>` field-name introspection.

**T02 — Curve evaluator:** Added pure `eval_scale(&ScaleCurve, f32) -> f32` and `eval_color(&ColorCurve, f32) -> [f32;4]` backed by a shared generic `eval_curve` helper. The helper sorts keyframe *indices* by `f32::total_cmp` (preserving authored order), clamps progress to [0,1], returns the first/last value at range boundaries, documents `DEFAULT_SCALE=1.0` / `DEFAULT_COLOR=[1,1,1,1]` for empty curves, and guards zero-span from duplicate `t` values. Added `resolve_effect<'a>(&'a VfxAsset, &str) -> Option<&'a EffectDef>` (None for absent ids) and `ImpactSpawnPlan { count, spread_px, ttl_ticks }` / `spawn_plan(&EffectDef)`. Eleven eval tests pass covering endpoints, midpoints, clamping, empty-curve defaults, single-keyframe constancy, and determinism (1000-call bit-identical check).

**T03 — vfx.ron asset:** Authored `assets/digimon/agumon/vfx.ron` with two effects: `baby_flame.impact` (count 8, spread_px 64.0, ttl_ticks 5 — reproducing the hardcoded `BABY_FLAME_IMPACT_SHARD_*` constants; ease-out scale curve sampled from `1-(1-t)²` at keyframes 0→0, 0.5→0.75, 1.0→1.0; orange-hue color with alpha fading 0.9→0.0) and `baby_flame.impact_flash` sibling (count 1, ttl 2, bright srgba 1.0/0.82/0.45 fading 0.95→0.0). Sibling chosen over on_expire chaining to keep the tracer bullet minimal. Four headless load tests pass via compile-time `include_str!`, asserting presence, spawn plan equality, and deterministic curve eval at sampled progresses. R012 re-confirmed: numeric appearance values live in the separate presentation asset; no numeric gameplay payload was added to any serialized command surface.

**T04 — Windowed glue:** Wired `src/windowed/render.rs` to source the Baby Flame impact fan-out from the data path. Registered `RonAssetPlugin::<VfxAsset>::new(&["ron"])` in `RenderPlugin::build` (windowed-gated); added `AgumonVfx { handle: Handle<VfxAsset> }` resource loaded at startup. `advance_vfx_particles` resolves `resolve_effect(asset, "baby_flame.impact")` once per frame; `Some` drives the data path (count/ttl from `spawn_plan`, outward distance from `spread_px * eval_scale(...)`, full rgba from `eval_color(...)`), `None` falls back to the hardcoded constants. Added `Option<u32>` ttl_override to `spawn_vfx_particle` so data-driven effects override the per-kind default; existing call sites pass `None`. Failure visibility: `diagnose_agumon_vfx_load` warns once on target `"windowed.agumon_playback"` with effect id + path + reason on `LoadState::Failed`; a missing effect id also warns once in the per-tick branch. VfxParticleKind, vfx_particle_kind, and all other effect kinds are untouched (deferred to S02). Three windowed_only integration tests pass pinning the lib contract render.rs consumes. R016 confirmed: resolver/eval live in the headless lib; `cargo build` (headless) stays clean.

## Verification

All four slice-level verification checks passed in the current session:

| # | Command | Exit Code | Verdict |
|---|---------|-----------|---------|
| 1 | `cargo test --test animation` | 0 | 83 passed, 0 failed (includes vfx_asset_schema ×3, vfx_asset_eval ×11, vfx_asset_load ×4, all prior tests) |
| 2 | `cargo build --features windowed` | 0 | Compiles clean — windowed glue valid |
| 3 | `cargo test --features windowed --test windowed_only vfx_asset_impact` | 0 | 3 passed, 0 failed (spawn plan, scale curve, color curve) |
| 4 | `cargo build` | 0 | Headless build clean — R016 (no windowed dep leak) confirmed |

R004 (pure headless curve eval): 11 determinism tests green. R012 (no numeric gameplay payload in serialized command): no numeric values added to any Command/SpawnParticle surface. R016 (windowed/headless boundary): resolver and eval live in `src/animation/vfx_asset.rs` (headless lib); only RonAssetPlugin registration and Handle live windowed-side. Visual confirmation of the Baby Flame impact fan-out via `cargo winx` is human-only UAT per K001 (auto-mode must not open a window).

## Requirements Advanced

None.

## Requirements Validated

- R004 — eval_scale/eval_color are pure, render-free, headless-tested: 11 tests covering endpoints, midpoints, clamping, empty-curve defaults, single-keyframe constancy, and 1000-call determinism — all green in cargo test --test animation vfx_asset_eval

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

["VfxParticleKind enum and vfx_particle_kind string-match still exist in src/windowed/render.rs — intentionally deferred to S02", "Baby Flame charge and launch effects still use hardcoded paths — S02 scope", "The hardcoded BABY_FLAME_IMPACT_SHARD_* constants and baby_flame_shard_offset/alpha remain as the fallback path and will be removed when S02 completes the full Registry migration"]

## Follow-ups

None.

## Files Created/Modified

- `src/animation/vfx_asset.rs` — 
- `src/animation/mod.rs` — 
- `Cargo.toml` — 
- `tests/animation/vfx_asset_schema.rs` — 
- `tests/animation/vfx_asset_eval.rs` — 
- `tests/animation/vfx_asset_load.rs` — 
- `tests/animation.rs` — 
- `assets/digimon/agumon/vfx.ron` — 
- `src/windowed/render.rs` — 
- `tests/windowed_only/vfx_asset_impact_render.rs` — 
- `tests/windowed_only.rs` — 
