# INVENTORY — tests files unification

**Date:** 2026-05-21
**Branch:** master
**Working dir:** `tests/`

## Motivation

`tests/` contains **135 top-level `.rs` files**. Cargo emits one binary per top-level test file. With `default-features = ["dev"]` and `bevy/dynamic_linking` active, each test binary is ~37–40 MB → ~5 GB across all tests. The flat layout also blocked R003's intent (which already documents that the refactor is pending). Consolidating into per-scope harness binaries reduces binary count ~7× and total test artifacts ~85%.

## Scope (135 files → 19 harnesses)

Partition produced by per-file classifier agents (`subagent` parallel pass, one agent per file), then validated by a sweep pass on borderline cases. Final partition:

| # | Scope | N | Files |
|---|---|---|---|
| 1 | `animation` | 8 | anim_gameplay_command_forbidden, anim_graph_asset, anim_graph_parse, anim_player_fsm, anim_stance_asset, anim_validation, clip_atlas_parity, agumon_sharp_claws_asset |
| 2 | `assets_data` | 7 | add_new_digimon_isolation, data_skills_ron_bounce, data_skills_ron_roundtrip, data_skills_ron_validation, data_units_ron_canonical, data_units_ron_roundtrip, roster_catalog |
| 3 | `bootstrap_encounter` | 7 | bootstrap_spawn_composition, combat_cli_shared_surface, encounter_bootstrap_internals, encounter_bootstrap_windowed, encounter_e2e, party_validation, slot_index_tiebreak |
| 4 | `damage_resolution` | 8 | block_reaction_pipeline, combat_coherence, combat_damage_edge, combat_damage_matrix, combat_resolution_apply, damage_breakdown_log, dr_pipeline, intent_applier_canary |
| 5 | `effects_kernel` | 3 | cleanse_effect, heal_effect, revive_semantics |
| 6 | `status_effects` | 11 | buffs_internals, modifiers_internals, status_accuracy, status_amp_pipeline, status_bag_unit, status_blessed, status_multi_kind_coexist, status_observability_canon, status_refresh_max_dur, status_slowed_delay, stun_internals |
| 7 | `passives_infra` | 4 | passive_canon_support, passive_event_filters, passive_reactive_canon, passive_runner_internals |
| 8 | `digimon_kits` | 16 | agumon_baby_burner_reactive, battery_loop_kernel, bouncing_fire_off_baseline, dorumon_blueprint, dorumon_predator_runtime, holy_support_affordance, holy_support_mechanics, holy_support_resolution, holy_support_roster_contract, passive_kitsune_grace, patamon_blueprint_seam, patamon_revive, predator_loop_kernel, renamon_precision_runtime, tentomon_blueprint, twin_core |
| 9 | `blueprints_infra` | 3 | blueprint_signal_dispatcher, digimon_signal_registry, form_identity |
| 10 | `follow_up` | 3 | follow_up_chains, follow_up_triggers, follow_up_triggers_internals |
| 11 | `action_query` | 5 | action_affordance_consumers, action_affordance_query, cast_id_propagation, commander_flow, engine_legality_integration |
| 12 | `timeline` | 16 | boundary_contract, compiled_timeline_active_canon, compiled_timeline_boot_validation, compiled_timeline_builtin_validation, compiled_timeline_runtime_dispatch, compiled_timeline_runtime_skills, perhop_guard, pipeline_dispatch, runtime_runner_internals, timeline_chain_bolt_port, timeline_circuit_breaker, timeline_cue_barrier_pipeline, timeline_mode_parity, timeline_onturnstart_kills, timeline_two_clock_parity, timeline_validate_loop_internals |
| 13 | `target_shape` | 5 | combat_resolution_bounce, combat_resolution_targets, target_shape_aoe_and_blast, target_shape_bounce_chain, target_shape_truthfulness |
| 14 | `turn_economy` | 10 | combat_resolution_streak, energy_internals, resource_caps, sp_economy, sp_mechanics_internals, turn_advance_split, turn_system_av, turn_system_internals, ultimate_charge_unit, ultimate_meter |
| 15 | `tempo_toughness` | 6 | tempo_resistance, tempo_resistance_internals, toughness_categories, toughness_enemy_only, toughness_internals, triangle_matchup |
| 16 | `runtime_events_obs` | 13 | cast_rng_internals, combat_state_internals, deterministic_rng_contract, event_bridge_internals, event_filter_internals, event_stream, kernel_internals, observability_log_internals, registry_internals, runtime_builtins_internals, signal_bus_internals, unit_died_payload, validation_snapshot |
| 17 | `preview_ai` | 6 | enemy_ai, enemy_ai_internals, enemy_ai_preview, presentation_metadata_boundary, scenario_ttk, skill_preview |
| 18 | `invariants` | 2 | properties, status_paralyzed_skip |
| 19 | `windowed_only` | 2 | phase_strip_readonly, windowed_preview_cache |

**Total: 135 ✓**

## Companion artifacts to migrate

| Artifact | Current location | After refactor |
|---|---|---|
| Shared fixtures | `tests/common/{mod.rs, actions.rs, app.rs, apply.rs, constants.rs, damage_helpers.rs, events.rs, resolution_helpers.rs, units.rs}` | Unchanged on disk. Harnesses include via `#[path = "common/mod.rs"] mod common;`. Cases use `use crate::common::...`. |
| Insta snapshot | `tests/snapshots/follow_up_triggers__agumon_break_follow_up_uses_real_pilot_config.snap` | `tests/follow_up/snapshots/triggers__agumon_break_follow_up_uses_real_pilot_config.snap` (1 file) |
| Cargo `[[test]]` entries | `Cargo.toml:62–132` (18 entries) | Removed — replaced by harness auto-discovery |
| README | `tests/README.md` | Update to describe new harness layout |
| `.gsd/KNOWLEDGE.md` R003 | "aggregation refactor pending" | Update to current state |

## Dependencies & ordering constraints

1. **`tests/common/`** must remain reachable from every harness. Path-attribute include (`#[path = "common/mod.rs"] mod common;`) keeps the shared module's source location stable.
2. **`Cargo.toml [[test]]` entries** point at top-level paths that will move. Removing entries before the harness exists breaks the build; removing after is the safe order — done in the final cleanup wave.
3. **Snapshot file**: rename + relocate happens together with `follow_up_triggers` migration so the test can find its snapshot on first run after the move.
4. **Per-file edits**: 29 of 135 files contain `mod common;` at top level. Each becomes a `super` reference once nested under `tests/<scope>/`. Mechanical sed-style edit.
5. **`#![cfg(feature = "windowed")]`** at the file head of `phase_strip_readonly.rs` and `windowed_preview_cache.rs`: gate is per-file and survives the move; the surrounding harness `tests/windowed_only.rs` declares its `mod`s unconditionally — bodies compile out when the feature is off, harness binary stays empty in headless builds.

## Scope of changes

- **Files moved:** 135 (`git mv tests/X.rs tests/<scope>/X.rs`)
- **Files created:** 19 harness `.rs` (one per scope)
- **Files edited:** 29 case files (rewrite `mod common;` → `use crate::common::...`)
- **Files edited (other):** `Cargo.toml`, `.gsd/KNOWLEDGE.md`, `tests/README.md`, 1 snapshot rename

## Estimated savings (target/)

| Metric | Before | After (projected) |
|---|---|---|
| Test binaries emitted | 135 | 19 |
| Per-binary size (avg) | ~37–40 MB | ~45–60 MB (harness pulls slightly more code per binary) |
| Total test artifacts | ~5 GB | ~1.0–1.2 GB |
| Compile wall-time (full `cargo build --tests`) | ~43s baseline | Similar or marginally faster (fewer link steps amortizes well) |
