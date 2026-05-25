---
id: S02
parent: M004
milestone: M004
provides:
  - Registry-resolved placement-verb dispatcher for all five Baby Flame effects
  - PlacementExt axis in ExtRegistries
  - Load-time validate_effects pass
  - Headless grep-guard CI for VfxParticleKind/kind_from_name absence
requires:
  - slice: S01
    provides: VfxAsset resolver/eval API and RonAssetPlugin wiring
affects:
  []
key_files: []
key_decisions:
  - Placement verbs registered into the D031 Registry<E> PlacementExt axis via register_agumon_ext — no global fn registry, no static dispatch
  - PlacementParams typed as an enum variant per verb (not a string map) so Reflect introspectability and serde round-trip are structural, not stringly-typed
  - validate_effects runs at asset-load time, warns once per invalid (effect, verb) pair, and skips/despawns the affected particle — never panics in the render loop
  - on_expire chain driven entirely from the asset field, not a hardcoded burst branch in the dispatcher
  - Grep-guard test reads src/windowed/render.rs at compile time (src/ is git-tracked) so the CI-provable invariant lives in the headless test lane
patterns_established:
  - PlacementExt axis pattern: add a placement verb = write one pure fn + one register() call in the Digimon blueprint, no core change
  - Load-time validation warn-and-skip pattern: surface the first invalid (effect, verb) pair as a named warning, skip the particle, never panic
  - Headless grep-guard pattern for windowed-code invariants: read the source file at test time to assert symbol absence
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-25T11:11:53.463Z
blocker_discovered: false
---

# S02: Placement verbs in Registry + generic render dispatcher

**All five Baby Flame VFX effects now render through Registry-resolved placement verbs from data; VfxParticleKind, kind_from_name, and all per-kind helpers deleted from render.rs, confirmed by headless grep-guard test.**

## What Happened

S02 ported the full Baby Flame VFX pipeline (charge ember-swirl, fast launch, projectile, impact fan-out, impact_shard/flash) to the owned data-driven path and deleted every hardcoded VFX-kind dispatch from src/windowed/render.rs.

**T01 — PlacementExt axis + pure placement verbs:** Four placement verbs (converge_inward, arc_launch, fan_out, static_offset) were added as pure fns registered into the D031 Registry<E> PlacementExt axis via register_agumon_ext in the Agumon blueprint. All verb math is headless-testable: typed PlacementParams (Reflect+Serialize+Deserialize), no World/render types, 1000-call bit-identical determinism confirmed by `placement_verbs::verbs_are_bit_identical_across_1000_calls`. The axis stands up with `all_four_verbs_resolve_via_freshly_built_registries` and `default_registries_have_no_placements`.

**T02 — Schema extension + all five effects authored:** The VFX schema was extended with typed anchor (VfxAnchor enum), per-effect ttl/size/texture_key, and a projectile→impact on_expire chain. All five effects (charge, ember, projectile, impact, impact_shard/flash) were authored in assets/digimon/agumon/vfx.ron. Load-time validation (`validate_effects`) surfaces unresolvable verb ids and dangling on_expire refs as named warnings (not panics). Tests `validate_effects_names_an_unregistered_verb`, `validate_effects_names_a_dangling_on_expire`, and `validate_effects_accepts_the_real_asset` certify the validation path. Schema Reflect introspectability confirmed by `placement_is_reflectable_with_typed_params_and_anchor` and `appearance_is_reflectable_with_expected_fields`.

**T03 — Generic dispatcher replaces VfxParticleKind:** The windowed advance_vfx_particles system now takes Res<ExtRegistries> and resolves per-tick particle position and appearance through Registry-dispatched placement verbs read from vfx.ron. VfxParticleKind enum, kind_from_name/vfx_particle_kind fn, and all per-kind helper fns were deleted from render.rs. The projectile→impact chain fires through the data `on_expire` field, not a hardcoded burst. `vfx_asset_impact_render` windowed contract tests (7 tests) confirm every authored effect resolves, verbs yield anchored offsets, and the on_expire chain works.

**T04 — Headless grep-guard:** `render_no_vfx_kind_guard::render_rs_has_no_vfx_kind_dispatch` reads src/windowed/render.rs at test time and asserts the three banned symbols (VfxParticleKind, kind_from_name, vfx_particle_kind) are absent, making success criterion 2 CI-provable in the headless lane without a window.

## Verification

**cargo test --test animation** (exit 0): 101 tests passed. Key tests: `render_no_vfx_kind_guard::render_rs_has_no_vfx_kind_dispatch` (grep-guard CI), `placement_verbs::verbs_are_bit_identical_across_1000_calls` (determinism), `placement_verbs::all_four_verbs_resolve_via_freshly_built_registries` (Registry axis), `vfx_asset_load::agumon_vfx_contains_all_five_effects`, `vfx_asset_load::projectile_on_expire_chains_the_impact_burst`, `vfx_asset_load::validate_effects_names_an_unregistered_verb`, `vfx_asset_load::validate_effects_names_a_dangling_on_expire`, `vfx_asset_load::validate_effects_accepts_the_real_asset`, `vfx_asset_schema::placement_is_reflectable_with_typed_params_and_anchor`, `vfx_asset_schema::appearance_is_reflectable_with_expected_fields`, `vfx_asset_schema::all_authored_effects_round_trip`.

**cargo build** (exit 0): Headless build clean, no windowed deps leaked into lib.

**cargo build --features windowed** (exit 0): Windowed dispatcher compiles cleanly.

**cargo test --features windowed --test windowed_only** (exit 0): 32 tests passed. Key tests: `vfx_asset_impact_render::built_registry_resolves_all_authored_placement_verbs`, `vfx_asset_impact_render::every_effect_resolves_and_its_verb_is_registered`, `vfx_asset_impact_render::impact_effect_resolves_to_the_spawn_plan_the_burst_loop_consumes`, `vfx_asset_impact_render::projectile_on_expire_chains_the_impact_fan`, `vfx_asset_impact_render::resolved_verbs_yield_the_expected_anchored_offsets`.

Visual UAT (charge ember-swirl + fast launch rendering through Registry-placed verbs in cargo winx) is K001 — human-only, not certifiable by auto-mode.

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

None.

## Follow-ups

None.

## Files Created/Modified

- `src/animation/placement.rs` — Four pure placement verbs: converge_inward, arc_launch, fan_out, static_offset with typed PlacementParams (Reflect+Serialize+Deserialize)
- `src/animation/vfx_asset.rs` — Extended schema: VfxAnchor, per-effect ttl/size/texture_key, on_expire chain; validate_effects load-time validation; PlacementExt axis wiring
- `src/animation/mod.rs` — Exposed placement module
- `src/combat/runtime/registry.rs` — PlacementExt axis added to ExtRegistries
- `src/combat/blueprints/agumon/mod.rs` — register_agumon_ext registers all four placement verbs
- `src/windowed/render.rs` — VfxParticleKind enum, kind_from_name, vfx_particle_kind, and all per-kind helper fns deleted; advance_vfx_particles rewritten as generic Registry-resolved dispatcher
- `assets/digimon/agumon/vfx.ron` — All five Baby Flame effects authored with typed params, anchor, ttl, size, texture_key, and on_expire chain
- `tests/animation/placement_verbs.rs` — Placement verb unit tests: all four verbs, determinism 1000-call, Registry dispatch, param mismatch
- `tests/animation/vfx_asset_schema.rs` — Reflect introspectability tests for Placement and Appearance; round-trip tests
- `tests/animation/vfx_asset_load.rs` — Load integration tests: all five effects, on_expire chain, validate_effects pass/fail cases
- `tests/animation/render_no_vfx_kind_guard.rs` — Headless grep-guard: asserts VfxParticleKind, kind_from_name, vfx_particle_kind absent from render.rs
- `tests/animation.rs` — Registered new test modules
- `tests/windowed_only/vfx_asset_impact_render.rs` — Windowed contract tests: Registry resolves all authored verbs, every effect resolves, on_expire chain, anchored offsets
- `tests/windowed_only.rs` — Registered windowed test module
