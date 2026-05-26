# S04: bevy_enoki integration spike (one effect)

**Goal:** Wire bevy_enoki as a windowed-gated GPU 2D particle backend and prove the single Agumon `baby_flame.impact` effect renders from a `.particle.ron` asset through the existing `spawn_effect_by_id` seam — without moving any particle lifetime into the kernel/FSM timeline (D031/D032 untouched) — while a headless dep-gating test proves `bevy_enoki` and its render-stack transitive deps never enter the default build, and `cargo test` (headless + windowed) plus `cargo build --features windowed` stay green.
**Demo:** In cargo winx, one Agumon skill's impact VFX renders through bevy_enoki from a .particle.ron asset; cargo test stays green and the dep-gating test passes.

## Must-Haves

- 1) Headless `cargo build`/`cargo test` (default `dev` features) never compile `bevy_enoki`; a dep-gating test asserts `bevy_enoki` is absent from the headless resolved graph and present under `--features windowed`. 2) `EnokiPlugin` is registered windowed-gated inside `RenderPlugin::build`. 3) One `.particle.ron` asset exists for `baby_flame.impact` and its handle loads with a fail-loud diagnostic on `LoadState::Failed`. 4) `spawn_effect_by_id` spawns an enoki one-shot `(ParticleSpawner, ParticleEffectHandle, OneShot, Transform)` bundle at the resolved anchor for the `baby_flame.impact` id only; every other effect id stays on the quad path; `request_release`/`fire_kernel_cue` control flow is unchanged. 5) `cargo build --features windowed` and `cargo test --features windowed --test windowed_only` stay green.

## Proof Level

- This slice proves: Mixed contract + integration. Real runtime required: yes — `cargo build --features windowed` must compile the full enoki render stack and the windowed_only contract test exercises the authored wiring. Human/UAT required: no — visual sign-off that enoki looks better than the placeholder is deferred to S05 (K001). The spike's mechanical proof is: dep-gating test (headless absence + windowed presence), windowed build green, and a source-contract test pinning the EnokiPlugin registration + the one-effect spawn branch + untouched kernel-cue control flow.

## Integration Closure

Upstream surfaces consumed: `RenderPlugin::build` (next to `RonAssetPlugin::<VfxAsset>`), the `spawn_effect_by_id` seam (src/windowed/render.rs:1341) and its three call sites (on-enter node spawn ~931, on_expire chain ~1747, detonate ~1578), `anchor_base_xy`, `VFX_PARTICLE_Z`, the `AGUMON_IMPACT_EFFECT_ID = "baby_flame.impact"` constant, and the `diagnose_agumon_vfx_load` fail-loud pattern. New wiring introduced: `bevy_enoki` optional dep gated behind `windowed`; `EnokiPlugin`; an `AgumonEnokiVfx` resource holding `Handle<Particle2dEffect>`; a load + fail-loud diagnostic system; an enoki one-shot spawn branch in `spawn_effect_by_id` for the one impact id. What remains before the milestone is usable end-to-end: S05 migrates Sharp Claws, Baby Flame, and Baby Burner fully to enoki and secures K001 visual sign-off.

## Verification

- A windowed-gated diagnostic system (mirroring `diagnose_agumon_vfx_load`, target `windowed.agumon_playback`) warns exactly once if the `baby_flame.impact` `.particle.ron` handle reports `LoadState::Failed`, naming the effect — so a future agent sees a contextual warning rather than a silently-missing particle. No new headless runtime signals. Failure visibility for the dep-leak risk is the dep-gating test itself: a regression that pulls bevy_enoki into the headless graph fails the test with a named assertion.

## Tasks

- [x] **T01: Add bevy_enoki as a windowed-only dep and prove no headless leak with a dep-gating test** `est:1h`
  Why: The central S04 risk is dependency-graph leakage — bevy_enoki 0.6 hard-depends on the entire Bevy render stack (bevy_render, bevy_sprite_render, bevy_core_pipeline, bevy_camera, bevy_mesh, bevy_shader). If it is reachable from the default build, the headless dep-isolation requirements (R002/R005/R016) are violated and the headless build balloons. Retiring this risk FIRST, before any effect authoring, de-risks the whole spike.
  - Files: `Cargo.toml`, `tests/dependency_gating.rs`
  - Verify: cargo test --test dependency_gating

- [x] **T02: Register EnokiPlugin windowed-gated and load the baby_flame.impact .particle.ron handle with a fail-loud diagnostic** `est:1h30m`
  Why: enoki must render through the existing windowed render path. EnokiPlugin brings its own asset loader for `Particle2dEffect`, so the `.particle.ron` needs no RonAssetPlugin registration — but its handle must be loaded into a resource and its load failure surfaced loudly (mirroring the M004 `diagnose_agumon_vfx_load` pattern) so a missing/broken asset never silently spawns nothing.
  - Files: `src/windowed/render.rs`, `assets/digimon/agumon/baby_flame_impact.particle.ron`
  - Verify: cargo build --features windowed

- [x] **T03: Spawn the enoki one-shot at the spawn_effect_by_id seam for baby_flame.impact and pin the wiring with a windowed contract test** `est:1h30m`
  Why: This closes the spike's demo — one Agumon impact effect rendering through enoki — by replacing the quad-spawn body for the ONE `baby_flame.impact` id while leaving every other effect on the existing quad path and the kernel/FSM cue/barrier control flow exactly as-is (D031/D032). Intercepting inside `spawn_effect_by_id` itself covers all three call sites (on-enter node spawn, on_expire projectile->impact chain, detonate) uniformly.
  - Files: `src/windowed/render.rs`, `tests/windowed_only/enoki_impact_render.rs`, `tests/windowed_only.rs`
  - Verify: cargo test --features windowed --test windowed_only

## Files Likely Touched

- Cargo.toml
- tests/dependency_gating.rs
- src/windowed/render.rs
- assets/digimon/agumon/baby_flame_impact.particle.ron
- tests/windowed_only/enoki_impact_render.rs
- tests/windowed_only.rs
