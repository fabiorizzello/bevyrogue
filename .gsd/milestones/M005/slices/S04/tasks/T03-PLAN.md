---
estimated_steps: 3
estimated_files: 3
skills_used: []
---

# T03: Spawn the enoki one-shot at the spawn_effect_by_id seam for baby_flame.impact and pin the wiring with a windowed contract test

Why: This closes the spike's demo — one Agumon impact effect rendering through enoki — by replacing the quad-spawn body for the ONE `baby_flame.impact` id while leaving every other effect on the existing quad path and the kernel/FSM cue/barrier control flow exactly as-is (D031/D032). Intercepting inside `spawn_effect_by_id` itself covers all three call sites (on-enter node spawn, on_expire projectile->impact chain, detonate) uniformly.

Do: (1) In src/windowed/render.rs, thread the enoki handle into `spawn_effect_by_id` (render.rs:1341) by adding a parameter `enoki: Option<&AgumonEnokiVfx>`. After computing `base = anchor_base_xy(...)` (reusing the existing placement math), add an early branch: `if effect_id == AGUMON_IMPACT_EFFECT_ID { if let Some(e) = enoki { commands.spawn((ParticleSpawner(default-material), ParticleEffectHandle(e.0.clone()), OneShot::Despawn, Transform::from_xyz(base[0], base[1], VFX_PARTICLE_Z))); return 1; } }`. Keep the existing quad loop for all other ids. (2) Update the three call sites and their owning systems to obtain `AgumonEnokiVfx` (add `Option<Res<AgumonEnokiVfx>>` to the system params of `advance_agumon_presentation`, `advance_vfx_particles`, and `spawn_detonate_particles`) and pass it through. MUST NOT alter `barrier.request_release(...)` / `sprite.player.fire_kernel_cue()` or any FSM control flow — only what gets spawned for the one id changes. (3) Create `tests/windowed_only/enoki_impact_render.rs` (a `#![cfg(feature="windowed")]` source-contract test using `include_str!("../../src/windowed/render.rs")`, following the pattern of tests/windowed_only/vfx_windowed_contracts.rs) asserting: `EnokiPlugin` is added in `RenderPlugin::build`; `spawn_effect_by_id` branches on `AGUMON_IMPACT_EFFECT_ID` and spawns an enoki bundle (assert presence of `ParticleSpawner`, `ParticleEffectHandle`, and `OneShot`); and `fire_kernel_cue` + `request_release` still appear in render.rs (control-flow-untouched guard). (4) Register the new test module in tests/windowed_only.rs with a `#[path]` line matching the existing entries. Skills: rust-development, bevy-ecs-expert.

Done-when: `cargo test --features windowed --test windowed_only` passes including the new contract test, and the quad path is unchanged for all non-impact effect ids.

## Inputs

- `src/windowed/render.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/vfx_windowed_contracts.rs`
- `assets/digimon/agumon/baby_flame_impact.particle.ron`

## Expected Output

- `src/windowed/render.rs`
- `tests/windowed_only/enoki_impact_render.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed --test windowed_only
