---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T05: M002 boundary map + S09 closeout evidence bundle

Why: M002's hard acceptance includes an explicit producer→consumer boundary map and a closeout that ties together R009/R013 proofs (from S08), the boundary map, captured soak console output, and the frame-time comparison; the roadmap `## Boundary Map` stub is empty and is DB-rendered, so the content lands as a dedicated, reader-testable S09 artifact. Skills: write-docs (reader-test for someone with no M002 context), verify-before-complete. Do: write `.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md` as a table mapping producer subsystem → contract/data type → consumer subsystem for at least: kernel(skills.ron gameplay numbers)→timeline→anim-graph(opaque cmds); anim player→cue barrier→kernel resume; CombatEvent→§9 UI/HUD (read-only); SkillGraphRegistry(skill-id)→windowed player (incl. the hardcoded-constant constraint to lift for M003+); VFX opaque ParticleId→windowed consumer (validate-only). Every row must cite an enforcing test file that exists on disk (e.g. tests/timeline/boundary_contract.rs, tests/windowed_only/phase_strip_readonly.rs, tests/preview_ai/presentation_metadata_boundary.rs, tests/animation/skill_graph_mapping_extensibility.rs, tests/animation/vfx_handle_seam.rs). Then write `.gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md` bundling: references to S08-SUMMARY/UAT for R009/R013 (cite the passing test names, do not re-validate), a link to the boundary map, the captured soak console log, the frame-time comparison verdict, and the regression-guard command result. Done-when: both docs exist, every cited test path resolves on disk, and the boundary map contains all five required producer→consumer rows.

## Inputs

- `.gsd/milestones/M002/slices/S08/S08-SUMMARY.md`
- `tests/timeline/boundary_contract.rs`
- `tests/windowed_only/phase_strip_readonly.rs`
- `tests/preview_ai/presentation_metadata_boundary.rs`
- `tests/animation/skill_graph_mapping_extensibility.rs`
- `tests/animation/vfx_handle_seam.rs`
- `.gsd/milestones/M002/slices/S09/frame-time-comparison.md`
- `.gsd/milestones/M002/slices/S09/soak-console.log`

## Expected Output

- `.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md`
- `.gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md`

## Verification

bash -c 'set -e; M=.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md; C=.gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md; test -f "$M"; test -f "$C"; for t in tests/timeline/boundary_contract.rs tests/windowed_only/phase_strip_readonly.rs tests/preview_ai/presentation_metadata_boundary.rs tests/animation/skill_graph_mapping_extensibility.rs tests/animation/vfx_handle_seam.rs; do test -f "$t" || { echo "missing cited test $t"; exit 1; }; grep -qF "$t" "$M" || { echo "boundary map does not cite $t"; exit 1; }; done; for row in kernel timeline anim-graph cue CombatEvent SkillGraphRegistry ParticleId; do grep -qiF "$row" "$M" || { echo "boundary map missing row keyword $row"; exit 1; }; done; echo BOUNDARY_MAP_AND_CLOSEOUT_OK'

## Observability Impact

N/A — documentation/evidence assembly.
