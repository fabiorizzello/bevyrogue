---
id: T03
parent: S04
milestone: M005
key_files:
  - src/windowed/render.rs
  - tests/windowed_only/enoki_impact_render.rs
  - tests/windowed_only.rs
key_decisions:
  - Intercept inside spawn_effect_by_id itself (not at each call site) so all three spawn sites — on-enter node, projectile on_expire chain, detonate — route baby_flame.impact through enoki uniformly with one code path
  - Use ParticleSpawner::default() (resolves to ColorParticle2dMaterial, the only Default impl) plus OneShot::Despawn so the effect self-despawns and no particle lifetime enters the kernel/FSM timeline (D031/D032 untouched)
  - Pin the wiring with a source-contract test (include_str! of render.rs) rather than a runtime test, because render.rs lives in the bin crate and K001 forbids running the windowed binary in auto-mode
duration: 
verification_result: passed
completed_at: 2026-05-26T09:30:47.009Z
blocker_discovered: false
---

# T03: Routed the single baby_flame.impact id through a bevy_enoki OneShot::Despawn spawner at the spawn_effect_by_id seam and pinned the wiring with a windowed source-contract test

**Routed the single baby_flame.impact id through a bevy_enoki OneShot::Despawn spawner at the spawn_effect_by_id seam and pinned the wiring with a windowed source-contract test**

## What Happened

Threaded an `enoki: Option<&AgumonEnokiVfx>` parameter into `spawn_effect_by_id` (src/windowed/render.rs). After computing `base = anchor_base_xy(...)` (reusing the existing quad placement math), added an early branch: when `effect_id == AGUMON_IMPACT_EFFECT_ID` and the enoki handle is present, spawn `(ParticleSpawner::default(), ParticleEffectHandle(enoki.handle.clone()), OneShot::Despawn, Transform::from_xyz(base[0], base[1], VFX_PARTICLE_Z))` and `return 1`; every other effect id falls through to the unchanged quad loop. OneShot::Despawn keeps the burst fire-and-forget so no particle lifetime enters the kernel/FSM timeline (D031/D032 untouched).

Verified the bevy_enoki 0.6 API against the vendored crate source before coding: ParticleSpawner<T> is #[require(...)]-gated and Default is implemented only for ParticleSpawner<ColorParticle2dMaterial> (so ::default() resolves the material); ParticleEffectHandle(pub Handle<Particle2dEffect>) and OneShot::{Deactivate, Despawn} live in bevy_enoki::prelude. Added imports for OneShot, ParticleEffectHandle, ParticleSpawner.

Updated all four call sites and their three owning systems to obtain the resource via Option<Res<AgumonEnokiVfx>> and pass agumon_enoki_vfx.as_deref(): advance_agumon_presentation (on-enter node spawn and projectile launch — two calls), spawn_detonate_particles (detonate), and advance_vfx_particles (the projectile->impact on_expire chain — the path that actually emits baby_flame.impact). No barrier.request_release(...) / sprite.player.fire_kernel_cue() / FSM control flow was altered.

Created tests/windowed_only/enoki_impact_render.rs, a #![cfg(feature="windowed")] source-contract test (include_str! of render.rs, following vfx_windowed_contracts.rs) with three cases: (1) EnokiPlugin is registered; (2) within the sliced spawn_effect_by_id block, the branch on AGUMON_IMPACT_EFFECT_ID spawns ParticleSpawner + ParticleEffectHandle + OneShot AND the for i in 0..count quad loop still exists for other ids; (3) control-flow guard — fire_kernel_cue() and request_release( still appear in render.rs. Registered the module in tests/windowed_only.rs with a #[path] line.

## Verification

cargo test --features windowed --test windowed_only enoki passes all 3 new enoki_impact_render contract tests plus the T02 parse test (4 passed, 42 filtered out). cargo build --features windowed recompiles the bin crate (which owns render.rs — the integration test target only reads it via include_str!, so the binary build is what actually type-checks the spawn-seam edits) and finishes clean. Slice-level verification stays green: dependency_gating passes (2 passed — bevy_enoki still absent from the headless graph, R005), and the full default-feature cargo test headless suite passes across all harnesses (0 failures). K001 honored — windowed binary not run; visual confirmation of the rendered enoki impact is deferred to manual cargo winx.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only enoki` | 0 | pass | 2411ms |
| 2 | `cargo build --features windowed` | 0 | pass | 2080ms |
| 3 | `cargo test --test dependency_gating` | 0 | pass | 280ms |
| 4 | `cargo test` | 0 | pass | 3041ms |

## Deviations

Updated four call sites rather than three: advance_agumon_presentation contains two spawn_effect_by_id calls (on-enter node spawn and projectile launch), both threaded through. The three named systems are unchanged in count; this is just the per-call-site detail.

## Known Issues

No runtime visual was confirmed: K001 forbids running the windowed binary in auto-mode, so the rendered enoki baby_flame.impact burst must be verified by the user in cargo winx. The contract test proves the wiring is present in source and the binary type-checks, but not that the effect looks correct on screen.

## Files Created/Modified

- `src/windowed/render.rs`
- `tests/windowed_only/enoki_impact_render.rs`
- `tests/windowed_only.rs`
