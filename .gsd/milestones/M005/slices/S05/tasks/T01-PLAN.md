---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Generalize the enoki spawn seam to a per-effect handle map

Why: S04 wired enoki for exactly one hardcoded id (baby_flame.impact); S05 must route three contact bursts uniformly without duplicating the branch per id. Do: In src/windowed/render.rs (1) add const path strings for the two new assets next to AGUMON_ENOKI_IMPACT_PATH: AGUMON_ENOKI_SHARP_CLAWS_PATH = "digimon/agumon/sharp_claws_slash.particle.ron" and AGUMON_ENOKI_DETONATE_PATH = "digimon/agumon/baby_burner_detonate.particle.ron"; (2) change AgumonEnokiVfx to hold `handles: std::collections::HashMap<String, Handle<Particle2dEffect>>` (add `use std::collections::HashMap;` or fully-qualify); (3) in load_agumon_enoki_vfx build the map keyed by effect id — AGUMON_SHARP_CLAWS_EFFECT_ID->slash asset, AGUMON_IMPACT_EFFECT_ID->impact asset, AGUMON_DETONATE_EFFECT_ID->detonate asset — and keep the Startup INFO log (log the three paths); (4) replace the `if effect_id == AGUMON_IMPACT_EFFECT_ID { if let Some(enoki) = enoki { ... } }` block in spawn_effect_by_id with `if let Some(enoki) = enoki { if let Some(handle) = enoki.handles.get(effect_id) { commands.spawn((ParticleSpawner::default(), ParticleEffectHandle(handle.clone()), OneShot::Despawn, Transform::from_xyz(base[0], base[1], VFX_PARTICLE_Z))); return 1; } }`, leaving the quad loop untouched for every other id; (5) generalize diagnose_agumon_enoki_vfx_load to iterate the handle map with a `Local<HashSet<String>>` warned-set, emitting the existing per-id WARN (path + effect id) once per failed handle. Then update tests/windowed_only/enoki_impact_render.rs: the existing assertions key on `effect_id == AGUMON_IMPACT_EFFECT_ID` and `enoki.handle` — rewrite them to assert the map-lookup branch (`enoki.handles.get(effect_id)`, `ParticleSpawner`, `ParticleEffectHandle`, `OneShot`) routes the three ids and that `for i in 0..count` (the quad loop) still exists for non-enoki ids; keep the EnokiPlugin-registered and fire_kernel_cue/request_release control-flow assertions. Done when: the windowed_only contract test passes against the generalized seam. Constraint: do not touch any kernel cue / barrier / FSM control flow (D031/D032); only what spawns for a matched id changes.

## Inputs

- `src/windowed/render.rs`
- `tests/windowed_only/enoki_impact_render.rs`

## Expected Output

- `src/windowed/render.rs`
- `tests/windowed_only/enoki_impact_render.rs`

## Verification

cargo test --features windowed --test windowed_only
