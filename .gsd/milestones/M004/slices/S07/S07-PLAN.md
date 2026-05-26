# S07: Validation remediation close scope and signoff gaps

**Goal:** Close M004 validation-remediation gaps with a canonical closeout artifact, roadmap-visible boundary map, honest visual-UAT waiver or blocker disposition, and executable guards proving the documentation surface matches current code and tests.
**Demo:** After this: M004 validation can be rerun with scoped requirement coverage, an inline or canonical boundary map, variant seam disposition, S06 assessment evidence, and human visual UAT signed off or formally waived.

## Must-Haves

- M004's validation findings have one canonical answer surface that cites requirement scope, boundary map, variant seam disposition, S06 assessment/UAT evidence, and D037 additive-material rescope.
- `.gsd/milestones/M004/M004-ROADMAP.md` no longer says the boundary map is not provided and instead contains an inline compact boundary table or canonical summary with links.
- Stale S04 proof tokens are repaired so the doc guard matches current test names, especially `projectile_on_expire_chains_the_impact_then_flash_fan`.
- `docs/uat/M004-vfx-signoff.md` is either updated to formal WAIVED status with reviewer/date/evidence fields and no visual PASS claim, or the S07 closeout doc explicitly marks manual visual signoff as the remaining external blocker.
- Fresh verification passes without running `cargo winx` or `scripts/capture-windowed-m004-vfx.sh`.

## Proof Level

- This slice proves: Final-assembly documentation and validation proof. Real runtime required: no. Human/UAT required: no for formal waiver path, yes only for visual PASS. K001 forbids auto-mode from launching the windowed binary, so S07 must never claim a human-eye PASS unless an existing signed artifact already provides it.

## Integration Closure

Consumes completed S04 scope/boundary docs, S05 rendering acceptance, and S06 UAT/assessment artifacts; introduces no runtime wiring. The slice closes the M004 validation surface by making already-delivered contracts discoverable and machine-checkable. If no waiver is recorded, milestone completion remains blocked only on external human visual signoff.

## Verification

- Adds or repairs executable documentation guards that fail with specific missing-token/missing-disposition messages, making future validation drift visible without re-reading the whole milestone history.

## Tasks

- [x] **T01: Repair stale S04 documentation guard** `est:45m`
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
  - Files: `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`
  - Verify: python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py

- [x] **T02: Author canonical validation remediation closeout** `est:1h`
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
  - Files: `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`, `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
  - Verify: test -s .gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md

- [x] **T03: Canonicalize boundary map and UAT disposition** `est:1h 15m`
  ---
  estimated_steps: 8
  estimated_files: 4
  skills_used:
    - write-docs
    - grill-me
    - verify-before-complete
  ---
  Why: Validation currently sees `Boundary Map: Not provided.` in the roadmap and unresolved PENDING visual UAT statuses. S07 must make those statuses canonical and honest before milestone validation can be rerun.
  Do: Replace the roadmap boundary-map placeholder with a compact inline producer-to-consumer table that references the S04 boundary map and S07 remediation doc. Include rows for VfxAsset/schema/eval, placement/appearance registry, projectile/impact chain, Baby Burner detonate, Sharp Claws slash, HDR/Bloom overbright rendering proxy, variant selection future seam, and K001 visual-UAT boundary. Update `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md` only where current post-S05/S06 truth supersedes point-in-time statements: Sharp Claws and HDR/Bloom automated proof are delivered, strict additive is D037-deferred, and visual quality remains human-only unless waived. For `docs/uat/M004-vfx-signoff.md`, choose the honest closeout path based on existing content: if no human PASS evidence exists, record a formal `WAIVED` disposition for M004 visual UAT rather than PASS, with reviewer/date/evidence fields and a note that no windowed binary was launched by auto-mode. If the project policy rejects agent waiver in the artifact itself, leave per-skill PENDING and update S07 remediation as an explicit external blocker; do not fabricate PASS.
  Done-when: The roadmap no longer contains `Not provided.` under Boundary Map, boundary docs do not contradict S05/S06, and the UAT artifact has either formal WAIVED closure or an explicitly documented pending blocker. Negative tests: later S07 guard must fail if the roadmap placeholder remains or if UAT is both PENDING and claimed closed.
  Threat surface: none. Requirement impact: documents final assembly boundaries only; no runtime behavior changes.
  - Files: `.gsd/milestones/M004/M004-ROADMAP.md`, `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`, `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`, `docs/uat/M004-vfx-signoff.md`
  - Verify: test -s docs/uat/M004-vfx-signoff.md

- [x] **T04: Add S07 closeout guard and run fresh regression proof** `est:1h 30m`
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
  - Files: `.gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py`
  - Verify: python3 .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py

## Files Likely Touched

- .gsd/milestones/M004/slices/S04/verify_s04_docs.py
- .gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md
- .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md
- .gsd/milestones/M004/M004-ROADMAP.md
- .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md
- docs/uat/M004-vfx-signoff.md
- .gsd/milestones/M004/slices/S07/verify_s07_validation_remediation.py
