---
estimated_steps: 12
estimated_files: 1
skills_used: []
---

# T01: Repair stale S04 documentation guard

---
estimated_steps: 5
estimated_files: 1
skills_used:
  - write-docs
  - tdd
  - verify-before-complete
---
Why: The current historical S04 checker is the first known failing proof surface because it still expects the old Baby Flame windowed-only test token. S07 needs that guard green before adding closeout docs, otherwise validation can still reject the milestone as stale.
Do: Read the existing checker and update only stale/current-proof assertions. Replace the old `projectile_on_expire_chains_the_impact_fan` expectation with the current `projectile_on_expire_chains_the_impact_then_flash_fan` test name, keep existing S04 scope/boundary assertions intact where still true, and avoid broad rewrites that change S04's completed-slice meaning. If the guard checks Sharp Claws or HDR as pending, either relax those historical assertions or update them to point at the S05/S06 superseding artifacts rather than declaring them undelivered.
Done-when: The repaired S04 checker exits 0 and still fails clearly for missing required scope/boundary/test-token evidence. Negative test coverage is implicit in the checker's missing-token assertions; do not remove assertions just to make it pass.
Threat surface: none; local documentation checker only. Requirement impact: supports re-verification of inherited/local M004 constraints while not mutating global requirements.

## Inputs

- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only/vfx_rendering_acceptance.rs`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`

## Expected Output

- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`

## Verification

python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py

## Observability Impact

Keeps the historical doc guard as a precise drift detector for missing proof-token names and stale scope claims.
