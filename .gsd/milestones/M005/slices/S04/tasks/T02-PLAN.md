---
estimated_steps: 3
estimated_files: 2
skills_used: []
---

# T02: Register EnokiPlugin windowed-gated and load the baby_flame.impact .particle.ron handle with a fail-loud diagnostic

Why: enoki must render through the existing windowed render path. EnokiPlugin brings its own asset loader for `Particle2dEffect`, so the `.particle.ron` needs no RonAssetPlugin registration — but its handle must be loaded into a resource and its load failure surfaced loudly (mirroring the M004 `diagnose_agumon_vfx_load` pattern) so a missing/broken asset never silently spawns nothing.

Do: (1) In src/windowed/render.rs, add `EnokiPlugin` to `RenderPlugin::build` right next to `app.add_plugins(RonAssetPlugin::<VfxAsset>::new(&["ron"]))` (render.rs:361). (2) Add an `AgumonEnokiVfx` resource holding the `Handle<Particle2dEffect>` and a Startup system `load_agumon_enoki_vfx` (added alongside the existing `load_agumon_vfx` Startup system at render.rs:366) that does `asset_server.load("digimon/agumon/baby_flame_impact.particle.ron")`. (3) Add an Update diagnostic system `diagnose_agumon_enoki_vfx_load` mirroring `diagnose_agumon_vfx_load` (render.rs:509): one-shot `warn!` on target `windowed.agumon_playback` naming the `baby_flame.impact` effect if the handle reports `bevy::asset::LoadState::Failed`. (4) Author `assets/digimon/agumon/baby_flame_impact.particle.ron` — a single-shot impact burst in enoki's `Particle2dEffect` schema (a DIFFERENT schema from VfxAsset; consult bevy_enoki 0.6 docs/examples for the exact field set: spawn_amount, lifetime, emission shape, color/scale curves). Keep it a one-shot burst suitable for a contact flash. Skills: rust-development, bevy-ecs-expert. Note: confirm the bevy_enoki 0.6 API names (EnokiPlugin, Particle2dEffect, ParticleSpawner, ParticleEffectHandle, OneShot) against the actual crate via `cargo doc`/source under the windowed feature; the research lists these but verify before coding.

Done-when: `cargo build --features windowed` compiles with EnokiPlugin registered, the AgumonEnokiVfx resource and load/diagnostic systems wired, and the .particle.ron asset present and parseable by enoki's loader.

## Inputs

- `src/windowed/render.rs`
- `Cargo.toml`
- `assets/digimon/agumon/vfx.ron`

## Expected Output

- `src/windowed/render.rs`
- `assets/digimon/agumon/baby_flame_impact.particle.ron`

## Verification

cargo build --features windowed
