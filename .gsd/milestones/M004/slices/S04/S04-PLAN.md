# S04: Validation scope and boundary documentation remediation

**Goal:** Resolve M004 validation ambiguity by documenting the exact validation scope, producer to consumer boundaries, and S03 consumed contracts so later validation can distinguish delivered S01 to S03 evidence from pending S05 and S06 work.
**Demo:** After this: M004 states exactly which requirements it validates, the boundary map is populated, S03 declares its consumed S01 and S02 contracts, and validation no longer flags documentation or scope ambiguity.

## Must-Haves

- M004 has a dedicated validation scope artifact stating that there are no active global requirements currently owned by M004, and that R002, R004, and R005 references are inherited or local constraint labels unless canonical requirement records are added.
- The validation scope artifact separates re-verified supporting invariants from not-yet-validated visual quality, Sharp Claws, and HDR bloom or additive rendering scope.
- M004 has a dedicated producer to consumer boundary map artifact with rows for owned VFX asset schema, placement verb registry, AnimGraph cue to owned effect ids, effect chaining, variant selection, failure visibility, and K001 manual visual boundary.
- S03 summary metadata explicitly declares consumed S01 and S02 contracts without changing S03 delivery claims.
- Fresh executable doc verification confirms the new artifacts exist, reference real code and test surfaces, mention pending S05 and S06 scope, and S03 requires metadata is no longer empty.

## Proof Level

- This slice proves: Documentation and contract evidence proof. No real runtime or human UAT is required for S04 itself; this slice does not claim final visual signoff. Verification is a repository-local script that checks the remediation docs, cited source/test surfaces, and S03 dependency metadata. Runtime rendering and K001 human review remain deferred to S05 and S06.

## Integration Closure

Upstream surfaces consumed: S01/S02/S03 summaries, current M004 context and roadmap, .gsd/REQUIREMENTS.md, VFX asset and render code, and existing animation/windowed tests. New wiring introduced: none in runtime code; this slice adds validation artifacts and repairs completed-slice dependency metadata. Remaining before milestone completion: S05 must deliver or rescope Sharp Claws and HDR/additive rendering criteria, and S06 must capture or waive windowed visual signoff.

## Verification

- No runtime observability changes. The improvement is validation observability: future agents get explicit documentation of owned scope, pending scope, producer to consumer boundaries, and an executable doc checker that localizes stale or missing evidence.

## Tasks

- [x] **T01: Write validation scope artifact** `est:45m`
  Expected executor skills frontmatter: write-docs, grill-me, verify-before-complete.
  - Files: `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md`
  - Verify: test -s .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md

- [x] **T02: Write producer consumer boundary map** `est:1h`
  Expected executor skills frontmatter: design-an-interface, write-docs, tdd, verify-before-complete.
  - Files: `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md`
  - Verify: test -s .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md

- [x] **T03: Repair S03 dependency metadata and add doc checker** `est:45m`
  Expected executor skills frontmatter: write-docs, tdd, verify-before-complete.
  - Files: `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md`, `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`
  - Verify: python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py

## Files Likely Touched

- .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md
- .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md
- .gsd/milestones/M004/slices/S03/S03-SUMMARY.md
- .gsd/milestones/M004/slices/S04/verify_s04_docs.py
