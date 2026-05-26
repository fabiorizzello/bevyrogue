---
estimated_steps: 13
estimated_files: 2
skills_used: []
---

# T02: Author canonical validation remediation closeout

---
estimated_steps: 7
estimated_files: 2
skills_used:
  - write-docs
  - design-an-interface
  - grill-me
  - verify-before-complete
---
Why: The milestone validation report names several gaps that are now split across S04, S05, and S06 artifacts. A fresh-reader closeout artifact must answer each finding directly without overclaiming global requirement validation or human visual PASS.
Do: Create `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md` with a table mapping every validation finding to disposition, evidence paths, and whether it is an automated proof, accepted rescope, future-only seam, formal waiver, or external blocker. Explicitly state `.gsd/REQUIREMENTS.md` has Active requirements: 0 and that R004/R005/R006/R007/R008/R010/R011/R012/R014/R015/R016 were previously validated, so M004 re-verifies local/inherited constraints rather than claiming new global requirement ownership. Document variant selection as a deterministic future-consumer seam proven in S03, not a missing M004 runtime integration. Cite D037's accepted rescope: strict custom additive material deferred; HDR + Bloom + overbright channels are the S05 proxy. Cite S06 assessment/UAT files as existing evidence while preserving the no-live-visual-PASS boundary. Add or update a short S04 scope note only if necessary to mark S07 as the current canonical closeout surface.
Done-when: The closeout doc has explicit sections for Requirement scope, Boundary map, Variant seam, S06 evidence, D037 rendering rescope, UAT disposition, and Verification commands. It must not claim `cargo winx` was run by auto-mode.
Failure modes: If an expected artifact is missing, document it as a blocker and make the later S07 guard fail clearly rather than silently omitting it. Load profile: trivial local file reads.

## Inputs

- `.gsd/REQUIREMENTS.md`
- `.gsd/milestones/M004/M004-VALIDATION.md`
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`
- `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`
- `.gsd/milestones/M004/slices/S06/S06-UAT.md`
- `docs/uat/M004-vfx-signoff.md`

## Expected Output

- `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`
- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`

## Verification

test -s .gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md

## Observability Impact

Creates a single inspection surface for validation disposition, reducing ambiguity when future agents rerun milestone validation.
