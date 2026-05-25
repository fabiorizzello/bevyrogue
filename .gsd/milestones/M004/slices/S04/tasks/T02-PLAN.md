---
estimated_steps: 4
estimated_files: 1
skills_used: []
---

# T02: Write producer consumer boundary map

Expected executor skills frontmatter: design-an-interface, write-docs, tdd, verify-before-complete.

Why: The roadmap boundary map is empty, and validation needs concrete producer to consumer contracts with proof names rather than ad hoc prose. This task creates the dedicated M004 boundary map artifact following the accepted M002 precedent.

Do: Create .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md. Use a stable table schema with columns such as Boundary, Producer, Contract, Consumer, Proof, and Status. Populate rows for: owned VFX asset schema; placement verb registry; AnimGraph presentation cue to owned effect ids; effect chaining; variant selection seam; failure visibility and validation boundary; and K001 manual visual boundary. For each row, cite actual paths and existing test function names, including schema/eval/load/variant tests, render_no_vfx_kind_guard, and windowed_only vfx asset impact tests. State limits honestly: variant selection is a proven seam, not full gameplay unlock wiring; render.rs effect id constants and texture-key mapping are not VfxParticleKind dispatch; K001 visual quality remains human-only and pending S06.

Done when: The artifact names producer, consumer, contract, and proof for each required boundary and contains no claims that Sharp Claws, HDR/additive rendering, or visual signoff are complete.

## Inputs

- `.gsd/milestones/M004/slices/S01/S01-SUMMARY.md`
- `.gsd/milestones/M004/slices/S02/S02-SUMMARY.md`
- `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
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

- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`

## Verification

test -s .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md

## Observability Impact

Improves validation observability by giving future agents a contract table that maps each proof failure to the producer and consumer boundary likely responsible.
