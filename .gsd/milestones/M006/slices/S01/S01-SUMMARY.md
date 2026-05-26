---
id: S01
parent: M006
milestone: M006
provides:
  - Single enoki spawn path in spawn_effect_by_id (no quad fallback); enoki handle map keyed by all six Agumon effect ids with per-id placement anchor; enoki lifecycle layer (ChargeEmberEnokiMarker, ProjectileFlight, advance_enoki_projectiles) for charge/ember/projectile behavioral semantics.
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - EnokiEffect { handle, anchor } struct (T02): modeled the per-id map value to carry both handle and PlacementAnchor, eliminating VfxAsset dependency from the enoki path before the loader was deleted in T04. Field name 'handles' preserved so existing source-contract test syntax stayed valid.
  - spawn_effect_by_id asset param widened to Option<&VfxAsset> (T03): required so the enoki impact chain in advance_enoki_projectiles can fire without a vfx.ron loader — forward-compatible with T04's loader deletion and confined to the windowed module.
  - Lifecycle dispatch by effect_id inside spawn_effect_by_id (T03): charge/ember tagged ChargeEmberEnokiMarker (persistent, cleared on launch), projectile tagged ProjectileFlight (persistent, cleared on arrival), contact bursts OneShot::Despawn — policy co-located with the spawn site.
  - AGUMON_PROJECTILE_FLIGHT_TICKS = 5 (T03): matches the deleted quad projectile's ttl_ticks from vfx.ron to preserve identical flight timing.
  - Code-shaped token assertions in the quad-deletion test (T05): matched 'fn advance_vfx_particles', 'VfxParticle {', 'for i in 0..count' rather than bare substrings because those identifiers survive in historical comments — bare !contains would false-fail.
patterns_established:
  - Enoki lifecycle tagging pattern: spawn persistent emitters (not OneShot::Despawn) with a marker component carrying the unit_id or flight data; a dedicated per-tick system advances/clears them; the spawner entity is despawned when the lifecycle ends, leaving no residual emitters.
  - Source-contract tests as deletion guards: to prove a system or component is gone, assert code-shaped tokens are absent ('fn foo', 'Struct {', loop header) rather than bare identifier substrings that survive in comments.
observability_surfaces:
  - diagnose_agumon_enoki_vfx_load WARN on target windowed.agumon_playback: reports missing handles for all six effect ids at startup — survives T04 and now covers charge/ember/projectile.
  - trace! on windowed.agumon_playback at ChargeEmberEnokiMarker despawn (charge/ember cleared on launch) and at ProjectileFlight arrival/impact-chain — lets a future agent confirm the launch-clear and chain fired without running the binary.
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T11:00:17.580Z
blocker_discovered: false
---

# S01: Enoki as sole VFX renderer

**Retired the custom quad VFX system and made bevy_enoki the sole particle renderer for all six Agumon effects; the full Baby Flame sequence, Sharp Claws, and Baby Burner now render through enoki with a native lifecycle layer for the charge/ember/projectile chain.**

## What Happened

S01 delivered end-to-end across five tasks:

**T01** authored the three missing Baby Flame sequence enoki assets (`baby_flame_charge.particle.ron`, `baby_flame_ember.particle.ron`, `baby_flame_projectile.particle.ron`) — all 19 Particle2dEffect fields explicit per MEM098. Charge and ember are continuous emitters (spawn_rate > 0) with `relative_positioning Some(true)` so they track the moving mouth anchor; the projectile is also continuous with `relative_positioning Some(false)` so it leaves a world-space trail as the flight system moves the spawner. Ember uses an enoki Attractor at origin to replicate the quad path's `converge_inward` behavior. Three new parse-contract tests (plus an `assert_continuous_emitter` helper) were added to `enoki_skill_effects_parse.rs`.

**T02** extended the `AgumonEnokiVfx` map value from a bare handle to an `EnokiEffect { handle, anchor }` struct and registered all six effect ids (`baby_flame.charge/ember/projectile/impact`, `baby_burner.detonate`, `sharp_claws.slash`) with anchors migrated from `vfx.ron`. The enoki branch in `spawn_effect_by_id` was moved above `resolve_effect` and now computes placement purely from the map entry, removing all `VfxAsset` dependency from the enoki path ahead of T04's loader deletion. Three new ids were also added to `enoki_effect_path` so the diagnostics WARN covers all six.

**T03** implemented the enoki-native lifecycle layer (D046): `ChargeEmberEnokiMarker { unit_id }` and `ProjectileFlight { from_xy, to_xy, ticks_total, ticks_elapsed }` components; dispatch inside `spawn_effect_by_id` tags charge/ember as persistent emitters (cleared at launch via a query on `CueReleaseResult::Released`), the projectile as a persistent `ProjectileFlight` entity, and contact bursts as `OneShot::Despawn`. The `advance_enoki_projectiles` system lerps the projectile transform each tick and chains the impact burst on arrival, reproducing the old `on_expire` chain. `spawn_effect_by_id`'s asset param was widened to `Option<&VfxAsset>` so the enoki impact chain in `advance_enoki_projectiles` can call it without a vfx.ron loader.

**T04** deleted the entire custom quad VFX system from `src/windowed/render.rs`: `VfxParticle/VfxParticleTarget/VfxParticleSource`, `VfxVisuals` + `load_vfx_visuals` + `vfx_texture_handle`, windowed `AgumonVfx` + `load_agumon_vfx` + `diagnose_agumon_vfx_load`, `RonAssetPlugin::<VfxAsset>` registration, `advance_vfx_particles` + `decrement_vfx_ttl`, and the quad spawn loop in `spawn_effect_by_id`. `spawn_effect_by_id` now reduces to: look up `(handle, anchor)` in the map → compute base → spawn enoki with lifecycle tagging → return 1; an unmapped id returns 0. System parameter lists and `RenderPlugin::build` were trimmed of all deleted references; `sample_animation_ticks.before(...)` was re-pointed at `advance_enoki_projectiles` (the new chain head). Two windowed source-contract tests that pinned the quad loop were updated as a direct consequence of the deletion (quad-loop assertion inverted; setup_camera_block slice boundary moved from deleted `load_vfx_visuals` to `load_agumon_enoki_vfx`). The lib `src/animation/vfx_asset.rs` and `on_enter_effect_ids` survive untouched.

**T05** completed the source-contract inversion in `tests/windowed_only/enoki_impact_render.rs`: added `quad_vfx_system_is_fully_deleted_from_render_src` asserting `fn advance_vfx_particles`, `VfxParticle {`, and `for i in 0..count` are absent (using code-shaped tokens to avoid false-positive matches against historical comments); renamed the contact-burst handle-map test to `enoki_handle_map_is_keyed_by_all_six_agumon_ids` and broadened it to all six ids; added `enoki_lifecycle_layer_is_present` pinning `ChargeEmberEnokiMarker`, `ProjectileFlight`, and `fn advance_enoki_projectiles`. The `kernel_and_fsm_control_flow_remains_untouched` test (D031/D032) survived unchanged.

## Verification

Slice-level verification run after T05 completion:

1. `cargo build --features windowed` → exit 0, Finished dev profile, zero warnings — windowed build green.
2. `cargo test --features windowed --test windowed_only` → 54 passed, 0 failed — all windowed source-contract suites pass including the new quad-deletion, six-id map, and lifecycle-layer contract tests.
3. `cargo test --test dependency_gating` → 2 passed (`bevy_enoki_absent_from_headless_graph`, `bevy_enoki_present_in_windowed_graph`) — dep-gating proves no enoki/windowed leak into the headless build.
4. `cargo test` (full headless) → exit 0 — headless suite green, lib `VfxAsset`/`resolve_effect` data-contract tests unaffected.
5. `grep "fn advance_vfx_particles|VfxParticle {|for i in 0..count" src/windowed/render.rs` → no matches — quad system fully absent from render.rs source.
6. `grep -c "ChargeEmberEnokiMarker|ProjectileFlight|advance_enoki_projectiles" src/windowed/render.rs` → 13 — lifecycle layer present.
7. `grep -c` for all six effect id strings in render.rs → 11 — all six ids registered in the enoki map.

VFX visual quality is K001 manual (deferred per D043); auto-mode never runs the windowed binary.

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

["VFX visual quality is K001 manual and unverified by auto-mode — the charge orb, ember swirl, and projectile trail were tuned from quad-path conventions but a human must confirm the on-screen read before M006 is closed."]

## Follow-ups

["K001 manual sign-off: human must run cargo run --features windowed and visually confirm the Baby Flame sequence (charge orb → ember swirl → trail projectile → impact burst), Sharp Claws slash, and Baby Burner detonate all render correctly through enoki with no quad artifacts."]

## Files Created/Modified

- `assets/digimon/agumon/baby_flame_charge.particle.ron` — 
- `assets/digimon/agumon/baby_flame_ember.particle.ron` — 
- `assets/digimon/agumon/baby_flame_projectile.particle.ron` — 
- `src/windowed/render.rs` — 
- `tests/windowed_only/enoki_impact_render.rs` — 
- `tests/windowed_only/enoki_skill_effects_parse.rs` — 
- `tests/windowed_only/vfx_windowed_contracts.rs` — 
