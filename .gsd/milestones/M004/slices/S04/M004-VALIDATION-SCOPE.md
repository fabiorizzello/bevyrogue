# M004 Validation Scope and Boundary Map

Milestone M004 / Slice S04 / Task T01.

This artifact removes the validation ambiguity around M004 by stating **what S01-S04 do validate, what they only re-verify/support, what remains pending for S05/S06, and which producer→consumer boundaries own each proof**.

## Summary

M004 currently owns **no active global requirements** in `.gsd/REQUIREMENTS.md`. The file's coverage summary reports **Active requirements: 0**. That means M004 should **not** claim to validate a new global requirement unless new requirement records are added through the GSD requirement tools.

Within `M004-CONTEXT.md`, the labels `R002`, `R004`, and `R005` should therefore be read as **inherited or local constraint labels** for this milestone's work:

- `R002` = headless-first constraint
- `R004` = deterministic math constraint
- `R005` = windowed dependency-gating constraint

Those labels describe the standards M004 must obey, but they are **not new global requirement ids created by this milestone**. For validation purposes, S01-S04 are limited to:

1. documenting and proving the owned VFX data seam,
2. re-verifying deterministic headless VFX math and windowed dependency gating,
3. documenting the boundary between delivered automation evidence and manual/pending visual work.

S01-S04 do **not** validate:

- K001 manual visual signoff,
- Sharp Claws VFX delivery,
- HDR bloom / additive rendering delivery,
- any new global requirement record.

## Scope table

| Scope item | Status after S01-S04 | Validation classification | Evidence owner |
|---|---|---|---|
| New global requirements for M004 | None recorded | **Not in scope** until added through GSD requirement tools | `.gsd/REQUIREMENTS.md` coverage summary (`Active requirements: 0`) |
| Owned per-Digimon VFX asset seam | Delivered and documented | **Supporting / re-verified invariant**, not a new global requirement validation | `assets/digimon/agumon/vfx.ron`; `src/animation/vfx_asset.rs`; `tests/animation/vfx_asset_schema.rs::vfx_asset_round_trips_through_ron`; `tests/animation/vfx_asset_load.rs::validate_effects_accepts_the_real_asset` |
| No gameplay numeric payload in the VFX presentation seam | Still required and still preserved | **Supporting / re-verified invariant** | `assets/digimon/agumon/vfx.ron` comments; `tests/animation/vfx_handle_seam.rs::spawn_particle_has_no_numeric_gameplay_payload`; `tests/animation/vfx_variant_selection.rs` + `src/animation/vfx_asset.rs` keep `VfxContext` render-free |
| Headless deterministic VFX math | Delivered in S01-S03 | **Supporting / re-verified invariant** | `tests/animation/vfx_asset_eval.rs::eval_scale_is_deterministic_across_repeated_calls`; `tests/animation/vfx_asset_eval.rs::eval_color_is_deterministic_across_repeated_calls`; `tests/animation/vfx_variant_selection.rs::select_variant_is_deterministic_across_repeated_calls` |
| Windowed dependency gating | Still respected | **Supporting / re-verified invariant** | `src/windowed/render.rs`; `M004-CONTEXT.md` technical constraints; prior slice evidence cited in `S03-SUMMARY.md` (`cargo build`, `cargo build --features windowed`, `cargo test --features windowed --test windowed_only`) |
| S03 variant-selection seam + Baby Burner detonate enrichment | Delivered | **Slice-local delivery**, not manual visual acceptance | `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`; `tests/animation/vfx_variant_selection.rs::select_variant_maps_context_to_expected_effect`; `tests/animation/vfx_asset_load.rs::baby_burner_detonate_is_fan_out_burst_chaining_flash` |
| Removal of hardcoded VFX kind dispatch | Delivered | **Slice-local delivery** | `tests/animation/render_no_vfx_kind_guard.rs::render_rs_has_no_vfx_kind_dispatch`; `src/windowed/render.rs` |
| K001 manual visual signoff | Not delivered by S01-S04 | **Pending S06 / manual-only** | `K001` in `.gsd/KNOWLEDGE.md`; `M004-CONTEXT.md` final integrated acceptance |
| Sharp Claws VFX | Not delivered by S01-S04 | **Pending S05 or formal rescope** | `assets/digimon/agumon/vfx.ron` currently contains `baby_flame.*` and `baby_burner.*`, not `sharp_claws.*` |
| HDR bloom / additive rendering | Not delivered by S01-S04 | **Pending S05 or formal rescope** | `src/windowed/render.rs::setup_camera` currently spawns `Camera2d`; no S01-S04 evidence names HDR/bloom/additive implementation |

## Producer → consumer boundary map

The milestone validator needs to know which subsystem owns each proof and where downstream consumers are allowed to depend on it.

| # | Producer | Contract | Consumer | Direction and constraint | Evidence owner |
|---|---|---|---|---|---|
| 1 | `assets/digimon/agumon/vfx.ron` + `src/animation/vfx_asset.rs` | Typed `VfxAsset` / `EffectDef` / `Placement` / `Appearance` / `variants` schema | Headless validation and windowed runtime loader | One-way authored-data seam: the asset owns effect ids and typed params; consumers read it, they do not reintroduce hardcoded kind dispatch | `tests/animation/vfx_asset_schema.rs::all_authored_effects_round_trip`; `tests/animation/vfx_asset_schema.rs::placement_is_reflectable_with_typed_params_and_anchor` |
| 2 | `src/combat/blueprints/agumon/mod.rs::register_agumon_ext` + `src/combat/runtime/registry.rs` | Registered `PlacementExt` verbs keyed by id | `src/windowed/render.rs::advance_vfx_particles` | One-way registry seam: render resolves a placement verb by authored id and applies it; verb math stays pure and headless-testable | `tests/animation/placement_verbs.rs::all_four_verbs_resolve_via_freshly_built_registries`; `tests/windowed_only/vfx_asset_impact_render.rs::built_registry_resolves_all_authored_placement_verbs` |
| 3 | `anim_graph.ron` `SpawnParticle` command names bridged in `src/windowed/render.rs::on_enter_effect_ids` | Opaque presentation cue names mapped to owned effect ids | Windowed spawn sites in `advance_agumon_presentation` | One-way presentation bridge: anim-graph cues do not carry numeric gameplay payload; they only name a presentation effect that resolves through `VfxAsset` | `tests/animation/render_no_vfx_kind_guard.rs::render_rs_has_no_vfx_kind_dispatch`; `src/windowed/render.rs` unit test `on_enter_charge_seeds_both_the_orb_and_the_ember_swirl` |
| 4 | `VfxAsset.on_expire` authored chains | Effect expiry triggers next owned effect id | Windowed particle lifecycle | One-way authored chain: follow-up effects come from data, not hardcoded projectile or detonate branches | `tests/animation/vfx_asset_load.rs::projectile_on_expire_chains_the_impact_burst`; `tests/animation/vfx_asset_load.rs::baby_burner_detonate_is_fan_out_burst_chaining_flash`; `tests/windowed_only/vfx_asset_impact_render.rs::projectile_on_expire_chains_the_impact_fan` |
| 5 | `src/animation/vfx_asset.rs::select_variant` + `VfxContext` + `VfxAsset.variants` | Pure variant selection `(skill_id, variant_key) -> effect id` | Future selection callers; S03 proves the seam only | One-way pure selection seam: no gameplay unlock system is introduced here; the selector remains deterministic and render-free | `tests/animation/vfx_variant_selection.rs::select_variant_maps_context_to_expected_effect`; `tests/animation/vfx_variant_selection.rs::select_variant_returns_none_for_unmapped_keys` |
| 6 | `src/animation/vfx_asset.rs::validate_effects` | Named validation errors for unknown verb, dangling `on_expire`, dangling variant target | Asset loading / verification / diagnostics | One-way failure-visibility seam: invalid authored data is localized and named, not allowed to fail silently as a generic render issue | `tests/animation/vfx_asset_load.rs::validate_effects_names_an_unregistered_verb`; `tests/animation/vfx_asset_load.rs::validate_effects_names_a_dangling_on_expire`; `tests/animation/vfx_variant_selection.rs::validate_effects_names_a_dangling_variant_target` |
| 7 | Windowed runtime (`cargo winx`) | Human-visible effect quality | Milestone validation / user signoff | Manual-only boundary: visual quality cannot be CI-asserted and must not be claimed by S01-S04 automation | `K001` in `.gsd/KNOWLEDGE.md`; `M004-CONTEXT.md` states visual quality is manual-only |

## S03 consumed contracts from earlier slices

S03 did not stand alone. Its delivered work consumes the following already-shipped contracts from S01 and S02.

| Consumer slice | Upstream slice | Consumed contract | Why S03 depends on it |
|---|---|---|---|
| S03 | S01 | Typed `VfxAsset` schema, effect resolution helpers, deterministic appearance/curve evaluation, owned `assets/digimon/agumon/vfx.ron` load path | S03's `variants` map and Baby Burner enrichment both extend the same authored asset and resolver/eval path |
| S03 | S02 | `PlacementExt` registry axis, registered Agumon placement verbs, `validate_effects`, registry-resolved render dispatcher, hardcoded kind-dispatch removal | S03's `baby_burner.detonate` fan-out burst and flash reuse the existing registered verbs and the data-driven windowed consumer |

This dependency statement is part of the validation scope even though the original `S03-SUMMARY.md` frontmatter left `requires: []`. Validators should treat S03 as consuming S01 and S02 contracts when tracing milestone proof.

## Pending scope reserved for S05 and S06

The following items remain outside the automated proof delivered by S01-S04 and should not be counted as complete in milestone validation yet.

### Pending S05

- **Sharp Claws VFX delivery or formal rescope.**
  - Current evidence owner: `assets/digimon/agumon/vfx.ron` has no `sharp_claws.*` entries.
- **HDR bloom / additive rendering delivery or formal rescope.**
  - Current evidence owner: `src/windowed/render.rs::setup_camera` spawns `Camera2d`; S01-S04 do not provide a passing proof for HDR, bloom, or additive blending.

### Pending S06

- **K001 manual visual signoff for Baby Flame, Baby Burner, and Sharp Claws in the real windowed build.**
  - Auto-mode cannot run the windowed binary for signoff.
  - Any milestone claim that the effects “look good” remains incomplete until the manual evidence exists or is formally waived.

## Validator guidance

A fresh validator should evaluate M004 using the following rule:

- Count S01-S04 as proof of the **owned VFX seam, deterministic headless math, registry-resolved placement boundary, data-driven chaining, and documented pending visual scope**.
- Do **not** count S01-S04 as proof of **visual quality signoff, Sharp Claws completion, or HDR bloom/additive rendering**.
- Do **not** treat `R002`, `R004`, and `R005` labels inside `M004-CONTEXT.md` as newly validated global requirement ids unless `.gsd/REQUIREMENTS.md` is explicitly updated through GSD requirement tooling.

That rule is the intended resolution of the M004 validation ambiguity.
