---
estimated_steps: 12
estimated_files: 1
skills_used: []
---

# T04: Add S07 closeout guard and run fresh regression proof

---
estimated_steps: 9
estimated_files: 1
skills_used:
  - tdd
  - write-docs
  - verify-before-complete
---
Why: S07 needs an executable current-closeout guard that checks all validation dispositions together, then fresh cargo proof that the documented data-driven VFX contracts still match code. This is the final proof before slice completion.
Do: Add `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`. The script should read only tracked repo artifacts and fail with specific messages if: the roadmap still says `Not provided.` for Boundary Map; the S07 remediation doc is missing requirement-scope, variant seam, D037, S06 assessment/UAT, or UAT disposition language; required evidence files are missing; stale Baby Flame token is used instead of `projectile_on_expire_chains_the_impact_then_flash_fan`; `docs/uat/M004-vfx-signoff.md` is still PENDING while S07 claims waiver/closure; or any doc claims auto-mode ran `cargo winx`. Then run the repaired S04 guard, the new S07 guard, and the VFX regression set. Do not run `cargo winx` or `scripts/capture-windowed-m004-vfx.sh`.
Done-when: Both documentation guards and all listed cargo commands exit 0. If visual UAT remains pending by policy, the S07 guard may exit nonzero with a clear external-blocker message; in that case do not complete the slice as full validation closure.
Failure modes: Missing docs or stale tokens must produce named failures. Load profile: cargo tests/checks only; no network or shared service. Negative tests: checker assertions for missing boundary placeholder removal, missing D037 citation, and forbidden auto-mode windowed-run claim.

## Inputs

- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`
- `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`
- `.gsd/milestones/M004/M004-ROADMAP.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`
- `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`
- `.gsd/milestones/M004/slices/S06/S06-UAT.md`
- `docs/uat/M004-vfx-signoff.md`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/windowed_only/vfx_rendering_acceptance.rs`

## Expected Output

- `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`

## Verification

python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py

## Observability Impact

Adds the primary machine-checkable S07 closeout diagnostic, with precise failure messages for stale validation surfaces.
