---
id: S04
parent: M004
milestone: M004
provides:
  - Validator-ready scope and boundary documentation for M004 S01-S04 delivery.
  - Executable regression check for S04 documentation integrity.
  - Explicit S03 upstream contract metadata for downstream milestone validation.
requires:
  - slice: S01
    provides: Typed `VfxAsset` schema and deterministic evaluation/load seams referenced by the repaired S03 consumed-contract metadata.
  - slice: S02
    provides: `PlacementExt` registry, verb registration, and render-dispatch seams referenced by the repaired S03 consumed-contract metadata.
  - slice: S03
    provides: Completed slice summary whose dependency metadata was repaired for downstream validation traceability.
affects:
  - S05
  - S06
key_files:
  - .gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md
  - .gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md
  - .gsd/milestones/M004/slices/S03/S03-SUMMARY.md
  - .gsd/milestones/M004/slices/S04/verify_s04_docs.py
key_decisions:
  - M004 currently owns no active canonical global requirements, so `R002`, `R004`, and `R005` references are documented as inherited/local labels rather than newly validated requirement records.
  - S03 consumed-contract dependencies should be recorded directly in summary frontmatter without altering S03's substantive delivery claims.
  - A token-based executable checker is the enforcement seam for documentation correctness, because it fails clearly when proof paths, cited test names, pending-scope statements, or dependency metadata drift.
patterns_established:
  - Use dedicated validation-scope and boundary-map artifacts to separate already-delivered evidence from intentionally deferred scope before milestone validation.
  - Backfill completed-slice dependency metadata in summary frontmatter when later validation needs explicit producer→consumer traceability.
  - Guard documentation-heavy remediation slices with a repository-local executable reader test so future edits cannot silently weaken validation evidence.
observability_surfaces:
  - `.gsd/milestones/M004/slices/S04/verify_s04_docs.py` provides a focused failure surface for missing artifacts, stale citations, removed pending-scope statements, and regressed dependency metadata.
drill_down_paths:
  - .gsd/milestones/M004/slices/S04/tasks/T01-SUMMARY.md
  - .gsd/milestones/M004/slices/S04/tasks/T02-SUMMARY.md
  - .gsd/milestones/M004/slices/S04/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-25T17:46:56.806Z
blocker_discovered: false
---

# S04: S04

**Closed M004 validation ambiguity by shipping dedicated scope/boundary artifacts, repairing S03 consumed-contract metadata, and proving the docs stay aligned with real proof surfaces via an executable checker.**

## What Happened

S04 finished the milestone-documentation remediation needed before later validation can judge M004 accurately. T01 authored `M004-VALIDATION-SCOPE.md` to state that M004 currently owns no active canonical global requirements, to classify `R002`, `R004`, and `R005` references as inherited or local constraint labels unless new requirement records are added, and to separate re-verified supporting invariants from still-pending visual signoff work. T02 added the dedicated `M004-BOUNDARY-MAP.md` artifact with validator-facing producer→consumer rows for the owned VFX asset schema, placement verb registry, AnimGraph cue-to-effect bridge, effect chaining, variant selection, failure visibility, and the K001 manual visual boundary, while explicitly recording non-claims for Sharp Claws and HDR bloom/additive rendering. T03 repaired `S03-SUMMARY.md` frontmatter so S03 now declares the S01 typed schema/eval/load seam and the S02 registry/validation/render-dispatch seam it consumed, without changing S03's delivery claims, and added `verify_s04_docs.py` as a repository-local reader test that checks artifact existence, required scope/boundary tokens, proof-file presence, representative cited test names, pending S05/S06 scope, and rejection of the old `requires: []` state. Together these changes turn the M004 validation gap into an explicit, executable contract for downstream S05/S06 and milestone validation.

## Verification

Ran `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py` through `gsd_exec`; it passed and confirmed the S04 scope/boundary docs, cited proof references, pending-scope statements, and repaired S03 dependency metadata remain consistent with on-disk sources and tests.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

This slice is documentation and verification only; runtime rendering acceptance, Sharp Claws scope resolution, HDR bloom/additive rendering decisions, and manual visual signoff remain for S05/S06.

## Follow-ups

S05 must deliver or formally rescope Sharp Claws plus HDR bloom/additive rendering criteria. S06 must capture or waive manual windowed visual signoff (K001).

## Files Created/Modified

- `.gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md` — Defines M004 validation ownership, inherited/local requirement-label interpretation, pending S05/S06 scope, and validator-facing scope boundaries.
- `.gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md` — Maps producer→consumer seams, proof citations, explicit consumed contracts, and non-claims for pending visual scope.
- `.gsd/milestones/M004/slices/S03/S03-SUMMARY.md` — Repairs `requires` frontmatter to declare the S01 and S02 contracts consumed by S03.
- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py` — Executable checker that validates artifact presence, required scope/boundary tokens, cited proof surfaces, pending-scope coverage, and repaired S03 dependency metadata.
