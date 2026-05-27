---
id: T01
parent: S08
milestone: M006
key_files:
  - assets/digimon/renamon/diamond_storm_leaf.particle.ron
  - src/windowed/digimon/renamon/mod.rs
  - tests/windowed_only/renamon_extension_contract.rs
  - tests/windowed_only/enoki_skill_effects_parse.rs
key_decisions:
  - ArcToTarget motion → EnokiLifecycle::Projectile (no Arc variant exists; on_arrival gracefully no-ops for diamond_storm.impact which is not yet registered)
  - Particle asset: L0 procedural, continuous emitter, world-space trail (relative_positioning: false), HDR green/white color scheme
  - Contract test renaming: S05 forbid-all → S08 forbid-only-unused (SkillReleaseEffectRegistry, DetonateEffectRegistry)
  - In-module enoki test uses TaskPoolPlugin + AssetPlugin + init_asset::<Particle2dEffect>() — pattern not previously established in codebase
duration: 
verification_result: passed
completed_at: 2026-05-27T08:46:03.983Z
blocker_discovered: false
---

# T01: Registered Renamon diamond_storm_leaf cue: authored particle asset + OnEnterEffectRegistry + EnokiVfxRegistry population + updated contract test + parse test

**Registered Renamon diamond_storm_leaf cue: authored particle asset + OnEnterEffectRegistry + EnokiVfxRegistry population + updated contract test + parse test**

## What Happened

T01 adds the full windowed VFX pipeline for Renamon's `diamond_storm_leaf` cue authored in `assets/digimon/renamon/anim_graph.ron` (line 9: `SpawnParticle(name: "diamond_storm_leaf", origin: CasterCenter, motion: ArcToTarget)`).

**Particle asset created** (`assets/digimon/renamon/diamond_storm_leaf.particle.ron`): L0 procedural primitive following the bevy-enoki-vfx skill guidance. Visual verb is a traveling projectile storm cloud. Colors: white-hot diamond glint at emission (HDR channels >1.0 for bloom: red:1.8, green:2.4, blue:1.6) → cool mint-green body → deep emerald fade. `relative_positioning: false` so shards leave a world-space trail behind the moving spawner. Continuous emitter (`spawn_rate: 0.012` ≈ 83/sec) with 5-tick flight duration matching Baby Flame's feel.

**renamon/mod.rs changes**: Added imports for `PlacementAnchor`, `EnokiEffect`, `EnokiLifecycle`, `EnokiVfxRegistry`, `OnEnterEffectRegistry`. Added two new Startup systems to `register(app)`:
- `register_renamon_on_enter_effects`: maps `"diamond_storm_leaf"` → `["diamond_storm.leaf"]` in `OnEnterEffectRegistry`
- `register_renamon_enoki_vfx`: loads the `.particle.ron` handle into `EnokiVfxRegistry` keyed by `"diamond_storm.leaf"` with `PlacementAnchor::CasterCenter` and `EnokiLifecycle::Projectile{flight_ticks: 5, on_arrival: "diamond_storm.impact"}`. The `on_arrival` id is intentionally not registered in S08 (graceful no-op on arrival).

**ArcToTarget → Projectile lifecycle decision**: `ArcToTarget` has no dedicated `EnokiLifecycle` variant. `Projectile` is the closest existing variant — it drives caster→target travel via `advance_enoki_projectiles` and chains `on_arrival` on arrival. The arc curve is not expressed in the lifecycle enum; a future slice can add an `Arc` variant without touching this registry entry (captured as MEM141).

**Contract test revised** (`tests/windowed_only/renamon_extension_contract.rs`): The S05 test `renamon_module_does_not_invent_fake_particle_or_engine_branches` previously asserted Renamon's mod.rs must NOT contain `EnokiVfxRegistry` or `OnEnterEffectRegistry`. S08 deliberately reverses this design call, so that test was renamed to `renamon_module_does_not_use_unused_registries` and now forbids only `SkillReleaseEffectRegistry` and `DetonateEffectRegistry` (genuinely unused). The positive assertion test was extended to include `"diamond_storm_leaf"`, `"diamond_storm.leaf"`, `"OnEnterEffectRegistry"`, and `"EnokiVfxRegistry"`.

**In-module tests added** (renamon/mod.rs `#[cfg(test)]`): Two new tests:
- `on_enter_diamond_storm_leaf_maps_to_the_owned_effect_id`: verifies the `"diamond_storm_leaf"` → `["diamond_storm.leaf"]` mapping in `OnEnterEffectRegistry`, including negative checks for near-miss names.
- `enoki_registry_holds_the_diamond_storm_leaf_entry`: verifies the `EnokiVfxRegistry` entry key, path, anchor, and lifecycle using `TaskPoolPlugin + AssetPlugin + init_asset::<Particle2dEffect>()`.

**Parse test added** (`tests/windowed_only/enoki_skill_effects_parse.rs`): `diamond_storm_leaf_parses_into_enoki_schema` verifies the new `.particle.ron` deserializes into `Particle2dEffect`, is a continuous emitter, uses `relative_positioning: Some(false)`, and has at least one HDR channel >1.0 at t=0.

## Verification

Ran `cargo test --features windowed --test windowed_only` (71 pass, 0 fail), `cargo test --features windowed --bin bevyrogue` (31 pass, 0 fail including 5 renamon in-module tests), and `cargo test --features windowed` full suite (all green). The revised contract test `renamon_module_does_not_use_unused_registries` and the new parse test `diamond_storm_leaf_parses_into_enoki_schema` both pass. Build clean: `cargo check --features windowed` exits 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features windowed` | 0 | Clean build, no warnings | 1500ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | 71 passed, 0 failed (was 70 before T01; +1 diamond_storm_leaf parse test) | 3000ms |
| 3 | `cargo test --features windowed --bin bevyrogue -- windowed::digimon::renamon` | 0 | 5 renamon in-module tests pass: register_inserts_stance_path, register_does_not_duplicate_stance_path, on_enter_diamond_storm_leaf_maps_to_owned_effect_id, enoki_registry_holds_diamond_storm_leaf_entry, register_populates_windowed_registries | 1000ms |
| 4 | `cargo test --features windowed` | 0 | Full suite: all tests pass (lib + bin + integration) | 5000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `assets/digimon/renamon/diamond_storm_leaf.particle.ron`
- `src/windowed/digimon/renamon/mod.rs`
- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/enoki_skill_effects_parse.rs`
