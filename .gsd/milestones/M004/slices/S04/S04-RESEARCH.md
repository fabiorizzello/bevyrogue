# S04 Research: Validation scope and boundary documentation remediation

## Summary

S04 is a documentation/evidence remediation slice, not a feature slice. The actual implementation work should clarify M004's validation scope, add a producer→consumer boundary map, and repair S03's dependency metadata so validation can distinguish delivered S01/S02/S03 contracts from still-pending S05/S06 visual/rendering scope.

Key finding: `.gsd/REQUIREMENTS.md` currently has **0 active requirements** and 11 already-validated requirements. M004 context uses labels like `R002/R004/R005` as inherited/local constraints (`headless-first`, `determinism`, `dep gating`), but those labels do **not** cleanly match the current global requirements file (`R004` and `R005` are already-validated historical requirements with different prose; there is no active `R002`). This is the main scope ambiguity to resolve in S04 docs: M004 should state whether it **validates no new global requirements** and instead **re-verifies/supports existing invariants** (especially VFX presentation seam/no gameplay payload and windowed gating), or whether new requirements must be recorded separately before validation.

## Skills Activated / Applied

Tool-level `Skill(...)` activation was not available in this harness schema, but the relevant installed skill guidance is directly applicable:

- `write-docs`: treat S04 artifacts as reader-testable docs for a fresh reviewer; every claimed boundary should name the producer, consumer, contract, and proof.
- `observability`: validation ambiguity should be resolved with explicit failure/verification surfaces; no hidden “looks good” claims without K001/manual evidence.
- `design-an-interface`: use a small explicit boundary-map schema instead of ad hoc prose; the contract columns are the interface.
- `grill-me`: stress-test the branch between “validated requirement” vs “supported/re-verified invariant” before updating requirements metadata.
- `verify-before-complete`: S04 completion should include fresh artifact checks (files exist, cited test names exist, docs contain all required rows), not just prose claims.

## Skills Discovered

Core technologies are Rust + Bevy + GSD documentation artifacts. Relevant skills are already installed in the prompt (`rust-development`, `rust-skills`, `bevy`, `write-docs`, `observability`, `review`, `verify-before-complete`). No external skill installation is needed.

## Current State

### Roadmap / docs

- `.gsd/milestones/M004/M004-ROADMAP.md` has `## Boundary Map` set to `Not provided.`
- `.gsd/milestones/M004/M004-CONTEXT.md` is rich, but requirement references are ambiguous:
  - `Relevant Requirements`: `R002 (headless-first)`, `R004 (determinism)`, `R005 (dep gating)`.
  - Current `.gsd/REQUIREMENTS.md` has active=0 and validated ids `R004`, `R005`, ... with different historical meanings; no active `R002` appears.
- Prior precedent from M002 S09: the roadmap Boundary Map section was DB-rendered/empty, so the team wrote a dedicated slice artifact `.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md` and validation accepted/cited it. Do the same for M004 unless a GSD tool explicitly supports editing the roadmap boundary map.

### Completed slice contracts

- S01 summary `provides`:
  - typed `VfxAsset` schema (Serialize + Deserialize + Reflect, deny_unknown_fields)
  - pure deterministic `eval_scale` / `eval_color` / `resolve_effect` / `spawn_plan`
  - initial `assets/digimon/agumon/vfx.ron` effects for `baby_flame.impact` and `baby_flame.impact_flash`
  - windowed data path + RonAssetPlugin/handle pattern
- S02 summary `requires` correctly points at S01. It `provides`:
  - Registry-resolved placement-verb dispatcher for all five Baby Flame effects
  - `PlacementExt` axis in `ExtRegistries`
  - load-time `validate_effects`
  - headless grep guard for `VfxParticleKind`/`kind_from_name` absence
- S03 summary currently has `requires: []`, but it consumed S01 and S02 contracts:
  - From S01: `VfxAsset` typed schema/eval/resolve API and `vfx.ron` asset path.
  - From S02: `PlacementExt` verbs, `validate_effects`, authored Baby Flame data path, and registry-resolved render contract.
  - S03 added variants + Baby Burner detonate/flash using existing verbs; it should declare those requirements explicitly.

### Current code/test surfaces to cite

Relevant files:

- `src/animation/vfx_asset.rs` — `VfxAsset`, effects/variants map, typed placement/appearance params, `eval_scale`, `eval_color`, `resolve_effect`, `select_variant`, `validate_effects`.
- `src/combat/runtime/registry.rs` — `ExtRegistries` and `PlacementExt` axis.
- `src/combat/blueprints/agumon/mod.rs` — Agumon placement verb registration via `register_agumon_ext`.
- `assets/digimon/agumon/vfx.ron` — current effect ids: `baby_flame.charge`, `baby_flame.ember`, `baby_flame.projectile`, `baby_flame.impact`, `baby_burner.detonate`, `baby_burner.flash`, `baby_flame.impact_flash`. No `sharp_claws.*` effect currently appears; that belongs to S05 or a formal rescope.
- `src/windowed/render.rs` — windowed consumer; no `VfxParticleKind` enum/string match remains, but it still has effect-id constants and texture-key mapping. These are documented as not kind dispatch.
- `tests/animation.rs` and `tests/windowed_only.rs` register the relevant test modules.

Useful test functions for boundary-map proof:

- Schema/editor readiness: `tests/animation/vfx_asset_schema.rs::{vfx_asset_round_trips_through_ron, all_authored_effects_round_trip, unknown_field_is_rejected, unknown_placement_field_is_rejected, appearance_is_reflectable_with_expected_fields, placement_is_reflectable_with_typed_params_and_anchor}`.
- Deterministic curves/resolution: `tests/animation/vfx_asset_eval.rs::{eval_scale_is_deterministic_across_repeated_calls, eval_color_is_deterministic_across_repeated_calls, resolve_effect_and_spawn_plan_read_appearance}`.
- Authored asset/chains/validation: `tests/animation/vfx_asset_load.rs::{agumon_vfx_contains_all_five_effects, projectile_on_expire_chains_the_impact_burst, baby_burner_detonate_is_fan_out_burst_chaining_flash, validate_effects_accepts_the_real_asset, validate_effects_names_an_unregistered_verb, validate_effects_names_a_dangling_on_expire}`.
- Variant seam: `tests/animation/vfx_variant_selection.rs::{select_variant_maps_context_to_expected_effect, select_variant_is_deterministic_across_repeated_calls, select_variant_returns_none_for_unmapped_keys, validate_effects_names_a_dangling_variant_target}`.
- Removal of hardcoded kind dispatch: `tests/animation/render_no_vfx_kind_guard.rs::render_rs_has_no_vfx_kind_dispatch`.
- Windowed registry/render contract: `tests/windowed_only/vfx_asset_impact_render.rs::{built_registry_resolves_all_authored_placement_verbs, every_effect_resolves_and_its_verb_is_registered, resolved_verbs_yield_the_expected_anchored_offsets, projectile_on_expire_chains_the_impact_fan}`.
- Render-unit cue/effect mapping: `src/windowed/render.rs` unit tests include `on_enter_charge_seeds_both_the_orb_and_the_ember_swirl`, `anchor_base_resolves_each_anchor_against_the_right_origin`, `skill_start_node_maps_each_bridged_skill_to_its_fsm_entry`, and `should_release_kernel_fires_on_authored_cue_frames`.

## Recommended Implementation Shape

### Task 1 — Write explicit M004 validation scope artifact

Create `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` (or similarly named artifact) with a short, unambiguous table:

| Scope item | Status in M004 | Evidence / owner |
|---|---|---|
| New global requirements | None currently active in `.gsd/REQUIREMENTS.md`; M004 should not claim new validation unless requirements are added via GSD requirement tools | `.gsd/REQUIREMENTS.md` coverage summary active=0 |
| Existing VFX presentation seam / no gameplay payload | Re-verified/supporting, not newly validated | `vfx_handle_seam`, `vfx_spawn_descriptor`, M004 vfx schema tests as supporting evidence |
| Headless-first deterministic VFX math | M004 contract criterion; S01-S03 automated evidence | `cargo test --test animation`, `vfx_asset_eval`, `placement_verbs`, `vfx_variant_selection` |
| Windowed dependency gating | Re-verified/supporting; windowed code/deps remain behind feature | `cargo build`, `cargo build --features windowed`, `Cargo.toml` windowed feature, tests |
| Visual quality / `cargo winx` signoff | Not validated by S01-S04; pending S06 or formal waiver | K001 manual-only |
| Sharp Claws VFX + HDR/bloom/additive | Not delivered by S01-S04; pending S05 or explicit rescope | current `vfx.ron` has no `sharp_claws.*`; `render.rs` has no bloom/additive evidence |

Important wording: call `R002/R004/R005` in M004-CONTEXT “inherited/local constraint labels” unless canonical requirement records exist. Avoid saying M004 validates global `R004/R005` unless `.gsd/REQUIREMENTS.md` is updated intentionally.

### Task 2 — Write M004 producer→consumer boundary map

Create `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md` following the M002 S09 precedent. Use rows like:

1. **Owned VFX asset schema** — Producer: `assets/digimon/agumon/vfx.ron`; Contract: typed `VfxAsset` with `EffectId`, `EffectDef`, `Placement`, `Appearance`, `variants`, Reflect/serde/deny_unknown_fields; Consumers: headless validators/evaluators and windowed asset loader. Proof: `vfx_asset_schema`, `vfx_asset_load` tests.
2. **Placement verb registry** — Producer: Agumon blueprint `register_agumon_ext` + `ExtRegistries.placements`; Contract: verb id resolves to pure `fn(&PlacementCtx, &PlacementParams) -> [f32;2]`; Consumer: `advance_vfx_particles` in `render.rs`. Proof: placement verb tests and `windowed_only/vfx_asset_impact_render::{built_registry_resolves_all_authored_placement_verbs,every_effect_resolves_and_its_verb_is_registered}`.
3. **AnimGraph presentation cue to owned effect ids** — Producer: `anim_graph.ron` `SpawnParticle` command names; Contract: opaque presentation particle names map to one or more owned effect ids via `on_enter_effect_ids`, not to `VfxParticleKind`; Consumer: `advance_agumon_presentation` / `spawn_effect_by_id`. Proof: `render_no_vfx_kind_guard`, render unit test `on_enter_charge_seeds_both_the_orb_and_the_ember_swirl`.
4. **Effect chaining** — Producer: `on_expire` fields in `vfx.ron`; Contract: particle expiry queues next authored effect id; Consumer: windowed particle lifecycle. Proof: `vfx_asset_load::projectile_on_expire_chains_the_impact_burst`, `vfx_asset_load::baby_burner_detonate_is_fan_out_burst_chaining_flash`, `windowed_only/vfx_asset_impact_render::projectile_on_expire_chains_the_impact_fan`.
5. **Variant selection seam** — Producer: `VfxContext { skill_id, variant_key }` + `VfxAsset.variants`; Contract: pure `select_variant` returns effect id or None; Consumer: future effect selection path (S03 proves seam, not full gameplay unlock wiring). Proof: `vfx_variant_selection::*` tests.
6. **Failure visibility / validation boundary** — Producer: `validate_effects(asset, known_verbs)`; Contract: deterministic first offending `VfxValidationError` for unknown verb/dangling on_expire/dangling variant; Consumer: windowed warn-and-skip behavior. Proof: `validate_effects_names_an_unregistered_verb`, `validate_effects_names_a_dangling_on_expire`, `validate_effects_names_a_dangling_variant_target`; code warning target `windowed.agumon_playback`.
7. **K001 manual visual boundary** — Producer: windowed runtime `cargo winx`; Contract: human visual quality signoff only; Consumer: milestone validation/S06. Proof: S01-S03 assessments mark visual checks `NEEDS-HUMAN`; not automatable.

Use M002 lesson P-L06: cite actual test function names, not only paths.

### Task 3 — Repair S03 consumed-contract metadata

Update `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md` frontmatter `requires` from `[]` to something like:

```yaml
requires:
  - slice: S01
    provides: Typed VfxAsset schema/eval/resolve API and owned assets/digimon/agumon/vfx.ron load path
  - slice: S02
    provides: PlacementExt registry axis, registered Agumon placement verbs, validate_effects, and registry-resolved windowed VFX data path
```

This is the direct remediation for “S03 declares its consumed S01 and S02 contracts.” Do not alter S03 delivery claims; just fix dependency metadata.

### Task 4 — Optional context/roadmap pointer

If permitted by GSD artifact conventions, add a short pointer in M004 docs saying the boundary map and validation scope are now in S04 artifacts. Because M002 S09 noted roadmap `## Boundary Map` was DB-rendered, avoid direct roadmap edits unless the planner confirms the safe writer/tool path.

A minimal direct edit target, if allowed, is `.gsd/milestones/M004/M004-CONTEXT.md` around “Relevant Requirements” to clarify canonical/global vs local labels. If using artifact-only remediation, make `M004-VALIDATION-SCOPE.md` explicit enough that validation can cite it without needing context edits.

## Natural Seams / Independent Work Units

- **Scope artifact** and **boundary map artifact** can be drafted independently after agreeing on the exact validation wording.
- **S03 summary metadata repair** is a small surgical edit independent of the new artifacts.
- **Verification script/check** can be last: grep/cite checks that files exist, S03 `requires` is no longer empty, every cited test function resolves, and `M004-BOUNDARY-MAP.md` includes all required rows.

## First Proof / Highest-Risk Unblocker

First prove the requirement scope statement, because it decides whether S04 updates docs only or must create/update requirement records. The most likely correct statement from current evidence:

> M004 has no active global requirements to newly validate. It delivers milestone acceptance criteria and re-verifies/supports already-validated project invariants (VFX/presentation seam, determinism, and windowed gating). Visual quality and Sharp Claws/rendering tech remain pending S05/S06 or formal rescope.

This should be reviewed before editing `.gsd/REQUIREMENTS.md`; otherwise the executor may accidentally attach M004 evidence to the wrong historical `R004/R005` records.

## Verification Recommendations

Run fresh verification before claiming S04 complete:

```bash
# Docs/artifact existence and key content
python3 - <<'PY'
from pathlib import Path
root = Path('.gsd/milestones/M004/slices/S04')
required = [root/'M004-VALIDATION-SCOPE.md', root/'M004-BOUNDARY-MAP.md']
for p in required:
    assert p.exists(), p
boundary = (root/'M004-BOUNDARY-MAP.md').read_text()
for needle in [
    'vfx_asset_schema', 'vfx_asset_load', 'built_registry_resolves_all_authored_placement_verbs',
    'render_rs_has_no_vfx_kind_dispatch', 'select_variant_maps_context_to_expected_effect',
    'K001', 'NEEDS-HUMAN'
]:
    assert needle in boundary, needle
s03 = Path('.gsd/milestones/M004/slices/S03/S03-SUMMARY.md').read_text()
assert 'slice: S01' in s03 and 'slice: S02' in s03
print('S04_DOCS_OK')
PY

# Ensure cited tests still compile/pass at the relevant granularity
cargo test --test animation vfx_asset_schema
cargo test --test animation vfx_asset_load
cargo test --test animation vfx_variant_selection
cargo test --test animation render_no_vfx_kind_guard
cargo test --features windowed --test windowed_only vfx_asset_impact_render
```

Full-suite commands are optional but useful if execution time allows:

```bash
cargo test --test animation
cargo build
cargo build --features windowed
cargo test --features windowed --test windowed_only
```

Do **not** run `cargo winx` in auto-mode; K001 requires human/manual visual signoff and belongs to S06 or a waiver.

## Watch-outs

- Do not mark Sharp Claws VFX, HDR bloom, additive blending, or visual signoff complete in S04. Current `assets/digimon/agumon/vfx.ron` has no `sharp_claws.*` effect, and S01-S03 assessments intentionally mark visual checks `NEEDS-HUMAN`.
- Do not treat the current `R004/R005` ids in `.gsd/REQUIREMENTS.md` as equivalent to the M004 context's shorthand labels without an explicit mapping decision.
- `src/windowed/render.rs` still has effect-id constants and texture-key mapping. The existing grep guard only forbids `VfxParticleKind`/`kind_from_name`; boundary docs should describe constants as spawn-site ids, not as closed kind dispatch.
- Prefer a dedicated S04 boundary-map artifact over editing `M004-ROADMAP.md` directly unless the planner confirms the GSD DB writer supports boundary-map content.

## Sources Consulted

- `.gsd/milestones/M004/M004-ROADMAP.md`
- `.gsd/milestones/M004/M004-CONTEXT.md`
- `.gsd/REQUIREMENTS.md`
- `.gsd/milestones/M004/slices/S01/S01-SUMMARY.md`
- `.gsd/milestones/M004/slices/S02/S02-SUMMARY.md`
- `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`
- `.gsd/milestones/M004/slices/S01/S01-ASSESSMENT.md`
- `.gsd/milestones/M004/slices/S02/S02-ASSESSMENT.md`
- `.gsd/milestones/M004/slices/S03/S03-ASSESSMENT.md`
- `.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md` precedent via repo search
- `assets/digimon/agumon/vfx.ron`
- `src/animation/vfx_asset.rs`, `src/windowed/render.rs`, `src/combat/runtime/registry.rs`
- Relevant tests under `tests/animation/*vfx*`, `tests/animation/render_no_vfx_kind_guard.rs`, `tests/windowed_only/vfx_asset_impact_render.rs`
