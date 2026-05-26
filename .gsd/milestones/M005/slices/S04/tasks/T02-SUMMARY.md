---
id: T02
parent: S04
milestone: M005
key_files:
  - src/windowed/render.rs
  - assets/digimon/agumon/baby_flame_impact.particle.ron
  - tests/windowed_only/enoki_impact_effect_parses.rs
  - tests/windowed_only.rs
key_decisions:
  - Rely on Bevy's by-asset-type loader selection so enoki's ParticleEffectLoader and RonAssetPlugin::<VfxAsset> coexist on the shared 'ron' extension without a custom extension
  - Author the .particle.ron with all 19 Particle2dEffect fields explicit (Option fields as Some(..)/None) because enoki's loader uses plain RON deserialize with no serde defaults
  - Add a windowed-gated ron::from_str parse test as durable proof of asset parseability, since cargo build does not exercise the asset loader and K001 forbids running the windowed binary in auto-mode
duration: 
verification_result: passed
completed_at: 2026-05-26T09:24:59.645Z
blocker_discovered: false
---

# T02: Registered EnokiPlugin windowed-gated and loaded the baby_flame.impact .particle.ron into AgumonEnokiVfx with a fail-loud one-shot diagnostic

**Registered EnokiPlugin windowed-gated and loaded the baby_flame.impact .particle.ron into AgumonEnokiVfx with a fail-loud one-shot diagnostic**

## What Happened

Verified the bevy_enoki 0.6 API against the vendored crate source before coding: EnokiPlugin, Particle2dEffect, EmissionShape, ParticleEffectHandle, ParticleSpawner, OneShot all live at the crate root / prelude as the plan listed. Confirmed the ParticleEffectLoader registers extension "ron" — the same as the existing RonAssetPlugin::<VfxAsset> — and that Bevy disambiguates by asset TYPE, so loading into Handle<Particle2dEffect> selects enoki's loader without conflict.

In src/windowed/render.rs: (1) added `.add_plugins(EnokiPlugin)` immediately after the RonAssetPlugin::<VfxAsset> registration, with a comment noting enoki brings its own loader and the windowed gate (R005/R016); (2) added the AGUMON_ENOKI_IMPACT_PATH const, an AgumonEnokiVfx resource holding Handle<Particle2dEffect>, and a Startup system load_agumon_enoki_vfx (wired alongside load_agumon_vfx) that asset_server.load(...)s the path and info!-logs the request on target windowed.agumon_playback; (3) added an Update diagnostic diagnose_agumon_enoki_vfx_load mirroring diagnose_agumon_vfx_load — a one-shot warn! (Local<bool> latch) naming the baby_flame.impact effect when the handle reports LoadState::Failed. No particle lifetime moved into the kernel/FSM timeline (D031/D032 untouched); the handle is merely held for a future spawn seam.

Authored assets/digimon/agumon/baby_flame_impact.particle.ron in enoki's Particle2dEffect schema — a one-shot ember burst (spawn_rate 0, spawn_amount 28, Circle(5.0) emission, ~0.32s lifetime, fast decelerating outward scatter, scale_curve shrinking to 0 and color_curve from hot yellow-white to transparent orange-red). All 19 fields are listed because the loader's RON deserialize has no field defaults.

Added a windowed-gated parse test tests/windowed_only/enoki_impact_effect_parses.rs (registered in the windowed_only harness, R003) that ron::from_str::<Particle2dEffect>(include_str!(...)) the git-tracked asset and asserts the one-shot/burst/lifetime/curve invariants — proving the asset is parseable by enoki's real schema, which a build alone does not check.

## Verification

cargo build --features windowed compiles green with EnokiPlugin registered and the AgumonEnokiVfx resource + load/diagnostic systems wired. The new windowed-gated test impact_effect_parses_into_enoki_schema passes, proving the .particle.ron deserializes into bevy_enoki::Particle2dEffect (same type render.rs loads). The dependency_gating tests still pass: bevy_enoki absent from the headless graph, present in the windowed graph — no render-stack dep leak (R005).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 65000ms |
| 2 | `cargo test --features windowed --test windowed_only impact_effect_parses_into_enoki_schema` | 0 | pass | 66000ms |
| 3 | `cargo test --test dependency_gating` | 0 | pass | 310ms |

## Deviations

Added tests/windowed_only/enoki_impact_effect_parses.rs (and its harness registration) beyond the plan's two expected output files. The plan's done-when requires the asset be "parseable by enoki's loader" but its stated verification was only cargo build, which does not load assets; the test closes that gap given the windowed binary cannot be run in auto-mode (K001).

## Known Issues

The handle in AgumonEnokiVfx is loaded and diagnosed but not yet attached to a ParticleSpawner — actual enoki rendering of baby_flame.impact is the next task's spawn-seam work. No runtime visual was confirmed (K001 forbids running the windowed binary in auto-mode; user must verify the VFX visually in cargo winx).

## Files Created/Modified

- `src/windowed/render.rs`
- `assets/digimon/agumon/baby_flame_impact.particle.ron`
- `tests/windowed_only/enoki_impact_effect_parses.rs`
- `tests/windowed_only.rs`
