---
id: S04
parent: M005
milestone: M005
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - Gate bevy_enoki strictly behind dep:bevy_enoki in the windowed feature — never in dev/default — to preserve R005/R016 headless isolation
  - Use cargo tree --invert --offline as the dep-gating regression guard, asserting on exit status + stdout-contains (not cargo's exact error string)
  - Dep-gating test lives as its own standalone headless binary (not under windowed_only/) so it runs on the default dev build
  - Rely on Bevy's by-asset-type loader selection: enoki's ParticleEffectLoader and RonAssetPlugin::<VfxAsset> coexist on the shared 'ron' extension without conflict
  - Author baby_flame_impact.particle.ron with all 19 Particle2dEffect fields explicit — enoki's loader uses plain RON deserialize with no serde defaults
  - Use source-contract tests (include_str! of render.rs) as the verification strategy for windowed-only wiring given K001 forbids running the windowed binary in auto-mode
  - Intercept inside spawn_effect_by_id itself (not at each call site) so all three spawn paths route uniformly through one branch
  - Use ParticleSpawner::default() + OneShot::Despawn to keep the burst fire-and-forget without touching kernel/FSM timeline (D031/D032 untouched)
patterns_established:
  - Dep-gating pattern: dep:<crate> optional feature + headless cargo tree --invert --offline test (MEM100)
  - Source-contract test pattern: include_str! of render.rs to pin windowed wiring without GPU context (MEM101)
  - Fail-loud diagnostic pattern: one-shot Local<bool> latch warn! naming the effect on LoadState::Failed, mirroring diagnose_agumon_vfx_load
observability_surfaces:
  - diagnose_agumon_enoki_vfx_load: one-shot WARN on target windowed.agumon_playback if baby_flame_impact.particle.ron handle reports LoadState::Failed
  - load_agumon_enoki_vfx: INFO log on target windowed.agumon_playback when the asset load is requested at Startup
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T09:33:18.363Z
blocker_discovered: false
---

# S04: S04: bevy_enoki integration spike (one effect)

**Wired bevy_enoki 0.6 as a windowed-only GPU 2D particle backend and routed the baby_flame.impact effect through a one-shot enoki spawner at the spawn_effect_by_id seam, with dep-gating and source-contract tests proving no headless dep leak and correct wiring.**

## What Happened

S04 retired the central spike risk (bevy_enoki's full render-stack leaking into the headless build) first, then wired the plugin and asset loading, and finally threaded the one-shot spawn seam.

**T01 — Dep-gating:** Added `bevy_enoki = { version = "0.6", optional = true }` to Cargo.toml and appended `"dep:bevy_enoki"` to the `windowed` feature list, leaving `dev`/`default` untouched. Created `tests/dependency_gating.rs` as a standalone headless test binary (not gated by `#![cfg(feature="windowed")]`) that shells out twice via `cargo tree --invert --offline`: once for the `dev` feature set (must exit non-zero / absent) and once for `windowed` (must exit 0 / present). Assertions key on exit status + stdout-contains rather than cargo's exact error string. Both tests passed immediately and have stayed green throughout the spike.

**T02 — Plugin + asset load:** Registered `.add_plugins(EnokiPlugin)` windowed-gated in `RenderPlugin::build` alongside the existing `RonAssetPlugin::<VfxAsset>`. Bevy disambiguates the shared `"ron"` extension by asset type, so enoki's `ParticleEffectLoader` and the VfxAsset loader coexist without conflict. Added an `AgumonEnokiVfx` resource holding a `Handle<Particle2dEffect>`, a `Startup` system `load_agumon_enoki_vfx` that calls `asset_server.load(AGUMON_ENOKI_IMPACT_PATH)` and logs the request at `windowed.agumon_playback`, and an `Update` diagnostic `diagnose_agumon_enoki_vfx_load` (one-shot `Local<bool>` latch) that `warn!`s by name if the handle reports `LoadState::Failed` — mirroring the M004 `diagnose_agumon_vfx_load` pattern. Authored `assets/digimon/agumon/baby_flame_impact.particle.ron` with all 19 `Particle2dEffect` fields explicit (enoki's loader has no serde defaults): a one-shot ember burst (spawn_rate 0, spawn_amount 28, Circle(5.0) emission, ~0.32 s lifetime, fast decelerating outward scatter, scale curve shrinking to 0, color curve from hot yellow-white to transparent orange-red). Added a windowed-gated parse test `enoki_impact_effect_parses.rs` that `ron::from_str::<Particle2dEffect>(include_str!(...))` the asset and asserts one-shot/burst/lifetime/curve invariants — closing the gap that `cargo build` alone does not exercise the loader.

**T03 — Spawn seam:** Threaded `enoki: Option<&AgumonEnokiVfx>` into `spawn_effect_by_id`. After computing the anchor position via `anchor_base_xy`, added an early branch: when `effect_id == AGUMON_IMPACT_EFFECT_ID` and the handle is present, spawns `(ParticleSpawner::default(), ParticleEffectHandle(enoki.handle.clone()), OneShot::Despawn, Transform::from_xyz(..., VFX_PARTICLE_Z))` and returns; every other effect id falls through to the unchanged quad loop. `OneShot::Despawn` keeps the burst fire-and-forget so no particle lifetime enters the kernel/FSM timeline (D031/D032 untouched). Updated all four call sites across three owning systems (`advance_agumon_presentation` — two calls for on-enter node spawn and projectile launch; `spawn_detonate_particles`; `advance_vfx_particles`) to obtain `Option<Res<AgumonEnokiVfx>>` and pass `agumon_enoki_vfx.as_deref()`. No barrier/cue/FSM control flow was altered. Pinned the wiring with `tests/windowed_only/enoki_impact_render.rs`, a source-contract test (include_str! of render.rs) with three assertions: EnokiPlugin registered, the AGUMON_IMPACT_EFFECT_ID branch spawns ParticleSpawner + ParticleEffectHandle + OneShot, and the quad loop still exists for other ids. Runtime visual confirmation of the rendered burst is deferred to the user via `cargo winx` (K001 forbids running the windowed binary in auto-mode).

**Deviations:** T02 added a windowed parse test not in the original plan, closing the loader-exercise gap. T03 updated four call sites rather than three (advance_agumon_presentation has two spawn_effect_by_id calls); the three named systems are unchanged in count.

## Verification

Fresh slice-level verification run at close-out:

1. `cargo test --test dependency_gating` → exit 0, **2 passed** (`bevy_enoki_absent_from_headless_graph`, `bevy_enoki_present_in_windowed_graph`). Proves R005/R016 dep-isolation invariant holds.
2. `cargo build --features windowed` → exit 0, **Finished** (incremental, already cached from T03). Proves the full enoki render stack compiles windowed-gated without errors.
3. `cargo test --features windowed --test windowed_only` → exit 0, **46 passed, 0 failed**. Includes: `impact_effect_parses_into_enoki_schema` (T02 parse test), `enoki_plugin_is_registered`, `spawn_effect_by_id_enoki_branch_spawns_correct_components`, `spawn_effect_by_id_quad_loop_unchanged_for_other_effects` (T03 contract tests), plus all prior S01–S03 windowed regression tests.
4. `cargo test` (headless default) → exit 0, **51 passed, 0 failed**. Full headless suite clean; no dep leak.

All four slice-level checks pass. Visual runtime confirmation of the enoki burst rendering is intentionally deferred to manual `cargo winx` (K001).

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Runtime visual confirmation of the enoki burst is deferred — K001 forbids running the windowed binary in auto-mode. S05 will migrate all three Agumon skills to enoki and secure K001 visual sign-off.

## Follow-ups

S05: migrate Sharp Claws, Baby Flame, and Baby Burner fully to enoki and secure K001 visual sign-off that they look better than the placeholder.

## Files Created/Modified

- `Cargo.toml` — Added bevy_enoki 0.6 as optional dep; appended dep:bevy_enoki to the windowed feature list
- `tests/dependency_gating.rs` — New standalone headless test: asserts bevy_enoki absent from dev graph and present in windowed graph via cargo tree --invert --offline
- `src/windowed/render.rs` — Registered EnokiPlugin; added AgumonEnokiVfx resource + load_agumon_enoki_vfx Startup system + diagnose_agumon_enoki_vfx_load fail-loud diagnostic; added enoki one-shot spawn branch in spawn_effect_by_id for AGUMON_IMPACT_EFFECT_ID; threaded Option<&AgumonEnokiVfx> through four call sites
- `assets/digimon/agumon/baby_flame_impact.particle.ron` — New Particle2dEffect asset: one-shot ember burst for baby_flame.impact with all 19 fields explicit
- `tests/windowed_only/enoki_impact_effect_parses.rs` — New windowed-gated parse test: ron::from_str::<Particle2dEffect> of the .particle.ron asset with invariant assertions
- `tests/windowed_only/enoki_impact_render.rs` — New windowed source-contract test: asserts EnokiPlugin registration, spawn branch wiring, and quad-loop preservation via include_str! of render.rs
- `tests/windowed_only.rs` — Registered enoki_impact_effect_parses and enoki_impact_render modules in the windowed_only harness
