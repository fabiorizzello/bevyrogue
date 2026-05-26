---
estimated_steps: 12
estimated_files: 4
skills_used: []
---

# T03: Canonicalize boundary map and UAT disposition

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

## Inputs

- `.gsd/milestones/M004/M004-ROADMAP.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`
- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md`
- `.gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md`
- `.gsd/milestones/M004/slices/S06/S06-UAT.md`
- `docs/uat/M004-vfx-signoff.md`

## Expected Output

- `.gsd/milestones/M004/M004-ROADMAP.md`
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
- `.gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md`
- `docs/uat/M004-vfx-signoff.md`

## Verification

test -s docs/uat/M004-vfx-signoff.md

## Observability Impact

Makes final boundary and UAT disposition visible from the roadmap plus a tracked signoff artifact instead of buried in slice history.
