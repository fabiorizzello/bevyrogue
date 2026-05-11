# S05: Shared-surface CLI proof

**Goal:** Reconcile existing M015/S05 completion evidence into the worktree DB.
**Demo:** After this: The CLI proves combat through shared action query, event, beat, snapshot, and kernel-observable surfaces, with no CLI-only combat path.

## Must-Haves

- Existing S05 CLI shared-surface proof is represented as complete in the DB.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Reconcile S05 completion evidence** `est:5m`
  Record existing M015/S05 artifact evidence as DB state. This is a reconciliation task only; no source changes are expected.
  - Files: `.gsd/milestones/M015/slices/S05/S05-SUMMARY.md`
  - Verify: test -f .gsd/milestones/M015/slices/S05/S05-SUMMARY.md

## Files Likely Touched

- .gsd/milestones/M015/slices/S05/S05-SUMMARY.md
