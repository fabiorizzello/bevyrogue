# Angle 1 — Agumon Enoki render path

## Question

If Agumon now uses Enoki particle assets, why are those VFX not visibly rendering in the windowed build?

## Findings

### 1. Asset registration and load request exist

`src/windowed/digimon/agumon/mod.rs` does register the full Agumon Enoki surface:
- six effect ids (`charge`, `ember`, `projectile`, `impact`, `sharp_claws`, `detonate`)
- on-enter particle-name -> effect-id mapping
- Baby Flame release-effect mapping
- detonate effect id

This means the failure is **not** “Agumon was never migrated to Enoki” or “no handles are loaded at all”.

### 2. Static contract/tests pass for the Agumon Enoki seam

Fresh run:
- `cargo test --features windowed --test windowed_only`

Relevant passing coverage from that suite:
- `enoki_impact_render::*`
- `enoki_skill_effects_parse::*`
- `vfx_windowed_contracts::*`
- `vfx_asset_impact_render::*`

What this proves:
- `EnokiPlugin` is registered
- Agumon particle assets parse
- the source still routes mapped ids through `spawn_effect_by_id`
- the expected Agumon ids are present in the registry contract

What it does **not** prove:
- that a live skill cast in the windowed runtime actually reaches `spawn_effect_by_id`
- that a spawned `ParticleSpawner` becomes visibly rendered during gameplay

### 3. Runtime validation logs show load requests, but no load-failure signal

Fresh windowed validation soak (`cargo run --bin bevyrogue --features 'dev windowed'` with validation env vars) logged:
- Agumon Enoki effects load requested
- no `diagnose_enoki_vfx_load` warning for failed `.particle.ron` assets

This narrows the failure surface:
- **unlikely:** missing/invalid Agumon `.particle.ron` asset
- **unlikely:** missing `EnokiPlugin`
- **unlikely:** missing Agumon registry population
- **still possible:** runtime trigger/spawn/visibility problem after load

### 4. The current automated coverage stops before the real failure seam

The current suite is very good at verifying:
- parsing
- ids
- registry shape
- source-structure contracts

But there is no proof that, after a real Agumon cast:
- a `ParticleSpawner` entity is spawned
- a projectile gets `ProjectileFlight`
- a one-shot burst is created at impact
- the effect is visible in the live render path

This means a regression can survive even with all current tests green.

### 5. The spawn path can fail silently at runtime

`spawn_effect_by_id` returns `0` when:
- the `EnokiVfxRegistry` resource is absent, or
- the `effect_id` is unmapped

The current call sites log this only at `trace!` level. So a runtime mismatch can easily look like “nothing rendered” without a strong warning in a normal run.

## Best current hypothesis

The Agumon issue is **probably not asset authoring or backend installation**. The highest-probability remaining breakpoints are:

1. **runtime trigger path** — the skill/action never reaches the spawn seam in the live session
2. **silent runtime mapping miss** — the effect spawn call executes but returns `0`
3. **live visibility/read issue** — the spawner exists, but the final effect is still not reading on screen

## Evidence

- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/render.rs::spawn_effect_by_id`
- `src/windowed/render.rs::diagnose_enoki_vfx_load`
- `assets/digimon/agumon/*.particle.ron`
- `cargo test --features windowed --test windowed_only` → 67 passed
- windowed validation soak log: Agumon Enoki load requested, no particle-load failure warning

## Risks / unknowns

- The soak run did not drive a live Agumon cast, so it cannot prove or falsify actual spawn timing.
- Current logs are too quiet when `spawn_effect_by_id` returns `0`.
- Visual read still needs a cast-driven windowed proof, not just load-time proof.

## Recommended next verification

1. Add a small cast-driven windowed smoke proof that asserts a `ParticleSpawner` appears for an Agumon skill cast.
2. Promote `spawn_effect_by_id == 0` from trace-only to a warn-once diagnostic per effect id / particle name.
3. If the spawner exists but the effect is still unreadable, inspect final render visibility/material parameters rather than the asset pipeline.

## Confidence

**Medium.** The evidence strongly rules out “no migration/no asset/no plugin”, but it does not yet isolate the final runtime break inside the live cast path.