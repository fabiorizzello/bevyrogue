---
estimated_steps: 6
estimated_files: 2
skills_used: []
---

# T03: Repair S03 dependency metadata and add doc checker

Expected executor skills frontmatter: write-docs, tdd, verify-before-complete.

Why: S03 currently declares requires as an empty list even though it consumed S01 and S02 contracts. This is the direct remediation item needed before validation can reason about cross-slice dependency closure. A small checker prevents the new documentation from silently drifting or omitting required evidence.

Do: Update only the S03 summary frontmatter dependency metadata so requires lists S01 and S02 with the consumed contracts. For S01, name the typed VfxAsset schema, eval/resolve API, and owned assets/digimon/agumon/vfx.ron load path. For S02, name the PlacementExt registry axis, registered Agumon placement verbs, validate_effects, and registry-resolved windowed VFX data path. Do not rewrite S03 delivery claims. Add .gsd/milestones/M004/slices/S04/verify_s04_docs.py as an executable repository-local verification script. The script should check that the two new S04 artifacts exist; required boundary headings/rows are present; pending items S05, S06, Sharp Claws, HDR/additive, and K001 are explicitly represented; cited source/test files exist; representative cited test names exist in their files; and S03 requires metadata names S01 and S02 and is not an empty list.

Failure Modes: If expected test names were renamed after planning, the checker should fail with a clear missing-token message so the executor updates docs to the current proof names rather than deleting proof. If a .gsd artifact path is missing, the checker should fail fast with the missing path. If frontmatter formatting differs, prefer a robust text/token check over a brittle YAML parser.

Negative Tests: The checker must reject missing required docs, missing required tokens, missing cited file paths, and the old `requires: []` S03 state.

Done when: S03 metadata documents its consumed S01 and S02 contracts and the checker passes from the repository root.

## Inputs

- `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `src/animation/vfx_asset.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `assets/digimon/agumon/vfx.ron`
- `src/windowed/render.rs`
- `tests/animation/vfx_asset_schema.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_variant_selection.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`

## Expected Output

- `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`
- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`

## Verification

python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py

## Observability Impact

Adds a deterministic documentation verification surface that future agents can run before milestone validation to locate missing or stale scope and boundary evidence.
