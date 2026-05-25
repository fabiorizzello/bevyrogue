# M004 Producer → Consumer Boundary Map

Milestone M004 / Slice S04 / Task T02.

This artifact is the dedicated producer→consumer contract table for M004. It complements `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` by isolating the exact **producer**, **consumer**, **contract**, **proof**, and **status** for each validation seam so later milestone validation can distinguish delivered S01-S03 evidence from the later S05/S06/S07 closeout dispositions.

## How to read this

A boundary here is a one-directional contract: the producer owns the data or behavior, and the consumer is allowed to depend only on that contract, not on the producer's hidden implementation details. For M004, the recurring rule is: **the owned Agumon VFX asset and pure lib-side helpers define the presentation contract; the windowed runtime consumes that contract without reviving hardcoded VFX-kind dispatch or pretending that automated proof equals live human visual review.**

## Boundary table

| # | Boundary | Producer | Contract | Consumer | Proof | Status |
|---|---|---|---|---|---|---|
| 1 | Owned VFX asset schema | `assets/digimon/agumon/vfx.ron`; `src/animation/vfx_asset.rs` | Typed `VfxAsset` / `EffectDef` / `Placement` / `Appearance` / `variants` schema; authored effect ids remain asset-owned data | Headless validation helpers and the windowed asset loader | `tests/animation/vfx_asset_schema.rs::all_authored_effects_round_trip`; `tests/animation/vfx_asset_schema.rs::placement_is_reflectable_with_typed_params_and_anchor`; `tests/animation/vfx_asset_load.rs::validate_effects_accepts_the_real_asset` | **Delivered** — schema and real asset are proven. The current asset now covers `baby_flame.*`, `baby_burner.*`, and `sharp_claws.slash`. |
| 2 | Placement verb registry | `src/combat/blueprints/agumon/mod.rs::register_agumon_ext`; `src/combat/runtime/registry.rs` | `PlacementExt` registry axis with registered pure verb ids (`static`, `converge_inward`, `arc_launch`, `fan_out`) | `src/windowed/render.rs::advance_vfx_particles` resolves verb ids from asset data through `ExtRegistries.placements` | `tests/windowed_only/vfx_asset_impact_render.rs::built_registry_resolves_all_authored_placement_verbs`; `tests/windowed_only/vfx_asset_impact_render.rs::every_effect_resolves_and_its_verb_is_registered` | **Delivered** — render consumes registered verb ids only; verb math remains lib-side and headless-testable. |
| 3 | AnimGraph presentation cue → owned effect ids | `src/windowed/render.rs::on_enter_effect_ids`; `src/windowed/render.rs` effect-id constants (`AGUMON_CHARGE_EFFECT_ID`, `AGUMON_EMBER_EFFECT_ID`, `AGUMON_PROJECTILE_EFFECT_ID`, `AGUMON_IMPACT_EFFECT_ID`, `AGUMON_DETONATE_EFFECT_ID`, `AGUMON_SHARP_CLAWS_EFFECT_ID`) | Opaque presentation cue names are bridged to owned effect ids, then resolved through `VfxAsset`; this is a cue-name mapping seam, not a gameplay payload seam | `advance_agumon_presentation`; `spawn_effect_by_id`; `spawn_detonate_particles` | `src/windowed/render.rs` unit test `on_enter_charge_seeds_both_the_orb_and_the_ember_swirl`; `tests/animation/render_no_vfx_kind_guard.rs::render_rs_has_no_vfx_kind_dispatch`; S05 Sharp Claws bridge proof in `tests/animation/vfx_asset_load.rs` | **Delivered with explicit limit** — `render.rs` still contains cue-name/effect-id constants and texture-key lookup, but it no longer uses `VfxParticleKind` / `kind_from_name` dispatch. This row does **not** claim fully data-driven cue registration. |
| 4 | Effect chaining (`on_expire`) | `assets/digimon/agumon/vfx.ron` `on_expire` fields; `src/animation/vfx_asset.rs` typed chain refs | Expiry of one owned effect id spawns the next owned effect id from data rather than a hardcoded render branch | Windowed particle lifecycle in `src/windowed/render.rs::advance_vfx_particles` | `tests/animation/vfx_asset_load.rs::projectile_on_expire_chains_the_impact_burst`; `tests/animation/vfx_asset_load.rs::baby_burner_detonate_is_fan_out_burst_chaining_flash`; `tests/windowed_only/vfx_asset_impact_render.rs::projectile_on_expire_chains_the_impact_fan` | **Delivered** — projectile→impact and detonate→flash chaining are proven as asset data. |
| 5 | Variant selection seam | `src/animation/vfx_asset.rs::select_variant`; `src/animation/vfx_asset.rs::VfxContext`; `assets/digimon/agumon/vfx.ron` `variants` map | Pure `(skill_id, variant_key) -> effect id -> EffectDef` selector over closed asset data | Future callers selecting an effect tree before render; S03 proves the seam only | `tests/animation/vfx_variant_selection.rs::select_variant_maps_context_to_expected_effect`; `tests/animation/vfx_variant_selection.rs::select_variant_is_deterministic_across_repeated_calls`; `tests/animation/vfx_variant_selection.rs::select_variant_returns_none_for_unmapped_keys` | **Delivered as a seam, not full feature wiring** — this proves deterministic selection and validation, but does **not** prove a real gameplay unlock system is wired end-to-end. |
| 6 | Failure visibility and validation boundary | `src/animation/vfx_asset.rs::validate_effects`; windowed warn-and-skip behavior in `src/windowed/render.rs::advance_vfx_particles` and `src/windowed/render.rs::diagnose_agumon_vfx_load` | Invalid authored data must fail with named validation errors or localized warnings (`UnknownVerb`, dangling `on_expire`, dangling variant target, missing loaded effect id) rather than as an unscoped render failure | Asset validation, diagnostics, and future milestone validation | `tests/animation/vfx_asset_load.rs::validate_effects_names_an_unregistered_verb`; `tests/animation/vfx_asset_load.rs::validate_effects_names_a_dangling_on_expire`; `tests/animation/vfx_variant_selection.rs::validate_effects_names_a_dangling_variant_target`; `tests/animation/render_no_vfx_kind_guard.rs::render_rs_has_no_vfx_kind_dispatch` | **Delivered** — proof localizes contract failures to the producer/consumer seam. This row does **not** certify live human visual quality. |
| 7 | K001 manual visual boundary | Project knowledge rule `K001` in `.gsd/KNOWLEDGE.md`; milestone scope doc `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`; `docs/uat/M004-vfx-signoff.md` | Human-visible quality in the windowed build remains outside auto-mode proof. Closure must be either a human signoff or an explicit tracked waiver; auto-mode must not claim it ran `cargo winx`. | Milestone validation / final acceptance | `.gsd/milestones/M004/slices/S01/S01-SUMMARY.md` (records `cargo winx` as human-only UAT); `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md` (explicitly defers K001 signoff); `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`; `docs/uat/M004-vfx-signoff.md` | **Closed via formal WAIVED disposition** — no live visual PASS is claimed, and the tracked waiver explicitly records that auto-mode did not launch the windowed binary. |

## S03 consumed contracts from earlier slices

S03's delivered work depends on earlier seams even though `S03-SUMMARY.md` frontmatter left `requires: []`.

| Consumer slice | Upstream slice | Consumed contract | Proof |
|---|---|---|---|
| S03 | S01 | Typed `VfxAsset` schema, `resolve_effect`, deterministic `eval_scale` / `eval_color`, and the owned `assets/digimon/agumon/vfx.ron` load path | `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`; `tests/animation/vfx_asset_load.rs::baby_burner_detonate_is_fan_out_burst_chaining_flash` |
| S03 | S02 | Registered `PlacementExt` verbs, `validate_effects`, registry-resolved render dispatcher, and removal of legacy VFX-kind dispatch | `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`; `tests/windowed_only/vfx_asset_impact_render.rs::every_effect_resolves_and_its_verb_is_registered`; `tests/animation/render_no_vfx_kind_guard.rs::render_rs_has_no_vfx_kind_dispatch` |

## Explicit limits for validators

These limits are part of the contract and should be preserved in any later validation pass:

- **Variant selection is proven only as a deterministic seam.** It is **not** proof that a real gameplay unlock/progression system selects variants in live combat.
- **`src/windowed/render.rs` still uses effect-id constants and texture-key mapping.** That is a cue/effect bridge and texture lookup surface, **not** a return to `VfxParticleKind` dispatch.
- **Sharp Claws and the HDR/Bloom rendering proxy are delivered by later slices, not by the original S04 proof set alone.** Read `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md` for the canonical post-S05 acceptance evidence.
- **Strict additive rendering is not delivered in M004.** D037 defers a custom additive material; the accepted delivered proxy is HDR + Bloom + overbright authored channels.
- **Live visual quality remains a human-only judgment boundary unless the tracked signoff artifact explicitly records a waiver.** Auto-mode must not claim the effects already looked correct in a real `cargo winx` session.

## Reader test

A fresh validator should be able to answer three questions from this file alone:

1. Which subsystem owns each VFX seam?
2. Which on-disk test will fail if that seam breaks?
3. Which gaps are still pending manual or later-slice work?

For M004, the answer is: **S01-S03 prove the authored asset/schema, registry-resolved placement, cue→effect bridge, data-driven chaining, variant-selection seam, and localized validation failures; S05 adds Sharp Claws plus the HDR/Bloom rendering proxy; D037 defers strict additive; and K001 visual closure must be either human signoff or a tracked waiver rather than an auto-mode PASS claim.**
