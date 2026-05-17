---
id: S14
parent: M021
milestone: M021
provides:
  - Truthful boundary evidence for slice validation and downstream closeout
  - A reusable add-new-digimon isolation regression
  - A documented narrowing of the no-Bevy and parity claims to the current tree
requires:
  - slice: S13
    provides: cast_id, UltInstant, and the 5-step turn pipeline proof that S14 builds on for boundary validation
affects:
  - S15
key_files:
  - tests/add_new_digimon_isolation.rs
  - .gsd/milestones/M021/slices/S14/tasks/T03-SUMMARY.md
  - .gsd/milestones/M021/slices/S14/tasks/T02-SUMMARY.md
  - .gsd/milestones/M021/slices/S14/tasks/T01-SUMMARY.md
key_decisions:
  - Keep the add-new-digimon claim at the extension boundary rather than editing shared runtime code.
  - Record the no-Bevy and parity claims truthfully as boundary evidence when the current tree does not yet satisfy the stronger statement.
patterns_established:
  - Use a dedicated integration regression to prove extension isolation without widening shared runtime edits.
  - Treat grep-backed boundary audits as first-class evidence for architecture claims when the codebase still violates the idealized target state.
observability_surfaces:
  - rg-backed structural audit and focused integration test output
drill_down_paths:
  - .gsd/milestones/M021/slices/S14/tasks/T01-SUMMARY.md
  - .gsd/milestones/M021/slices/S14/tasks/T02-SUMMARY.md
  - .gsd/milestones/M021/slices/S14/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-17T13:41:55.431Z
blocker_discovered: false
---

# S14: Prove two-clock parity and extension boundaries

**Recorded explicit proof that the add-new-digimon workflow stays owner-keyed and registry-isolated, while the blueprint boundary audit truthfully narrows the no-Bevy claim to the current shared modules that still import Bevy directly.**

## What Happened

T01 established that the requested parity filters currently match no tracked tests, so the slice does not overclaim HeadlessAuto-vs-Windowed intent-stream equivalence from the existing tree. T02 performed the blueprint boundary audit and found current shared blueprint modules still importing Bevy directly, so the original absolute no-Bevy claim had to be treated as a boundary check rather than a completed isolation guarantee. T03 added a git-tracked add-new-digimon regression test at tests/add_new_digimon_isolation.rs that proves existing roster metadata stays optional, unknown blueprint owners are rejected through the registry dispatch seam, and canonical blueprint signals remain owner-keyed; the test suite commands for add_new_digimon, blueprint, and the focused isolation test all passed. Together these task outcomes provide the truthful cross-slice boundary evidence needed for the slice artifacts and set up the remaining milestone closeout work in S15.

## Verification

Verified the focused add-new-digimon regression with cargo test --test add_new_digimon_isolation -- --nocapture, plus the broader cargo test -- --nocapture add_new_digimon and cargo test -- --nocapture blueprint runs, all of which exited 0. Also confirmed via the task summaries that the parity filter commands from T01 currently discover zero tests and the blueprint grep audit from T02 still surfaces Bevy imports in shared blueprint modules, which keeps the slice narrative truthful instead of overstating the current state.

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

The original absolute no-Bevy shared-boundary claim was narrowed to the truthful current state because the grep audit still finds Bevy imports in shared blueprint modules. The parity task was also recorded as discovery-bounded because the requested test filters currently match no tracked tests.

## Known Limitations

HeadlessAuto-vs-Windowed intent-stream equivalence is not yet proven by an actual tracked test in the current tree, and shared blueprint modules still depend directly on Bevy. The slice therefore documents boundary evidence rather than a complete architectural cleanup.

## Follow-ups

Complete the milestone closeout in S15 with fresh end-to-end verification, and decide whether the blueprint Bevy imports are acceptable shared-boundary dependencies or should be split in a future slice.

## Files Created/Modified

- `tests/add_new_digimon_isolation.rs` — Added a focused integration regression proving add-new-digimon isolation stays owner-keyed and roster metadata remains optional for existing units.
- `.gsd/milestones/M021/slices/S14/tasks/T03-SUMMARY.md` — Recorded the task-level proof and verification outcome for the add-new-digimon isolation regression.
- `.gsd/milestones/M021/slices/S14/tasks/T03-VERIFY.json` — Stored verification evidence for the focused regression and broader blueprint/add_new_digimon test runs.
- `.gsd/milestones/M021/slices/S14/S14-PLAN.md` — Roadmap checkbox for T03 is now reflected in the completed task state.
