---
id: S15
parent: M021
milestone: M021
provides:
  - Fresh runtime-closeout evidence for milestone validation.
  - Truthful artifact-level separation between green runtime proof and still-open architecture-boundary audits.
requires:
  - slice: S13
    provides: fresh remediation proofs for boot validation, DryRun/Execute parity, and boundary evidence
  - slice: S14
    provides: boundary and add-new-digimon evidence that S15 rolls up into final closeout
affects:
  - milestone validation for M021
key_files:
  - tests/compiled_timeline_boot_validation.rs
  - tests/follow_up_triggers.rs
  - tests/follow_up_chains.rs
  - tests/pipeline_dispatch.rs
  - .gsd/milestones/M021/slices/S15/tasks/T01-SUMMARY.md
  - .gsd/milestones/M021/slices/S15/tasks/T02-SUMMARY.md
key_decisions:
  - Separate runtime closeout proof from architecture-boundary grep audits so the slice stops overclaiming a fully closed milestone.
  - Fix stale test harnesses before drawing conclusions from the closeout battery.
patterns_established:
  - When timeline-backed skills are exercised in focused test apps, those harnesses must compile and install `TimelineLibrary<String>` and register both kernel builtins and blueprint-owned extensions.
  - Closeout slices should distinguish green runtime verification from architecture-boundary audits instead of collapsing both into a single pass/fail claim.
observability_surfaces:
  - none
drill_down_paths:
  - .gsd/milestones/M021/slices/S15/tasks/T01-SUMMARY.md
  - .gsd/milestones/M021/slices/S15/tasks/T02-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-17T14:11:02.780Z
blocker_discovered: false
---

# S15: Final milestone closeout evidence

**S15 restored green final runtime verification for M021, fixed stale timeline test harnesses, and recorded the remaining architecture-boundary grep hits explicitly so the slice can close truthfully.**

## What Happened

S15 re-established final closeout evidence for the integrated M021 tree without depending on stale summaries. T01 first repaired the failing boot-validation harness in `tests/compiled_timeline_boot_validation.rs`, then uncovered and fixed a broader regression in the legacy follow-up and pipeline tests: timeline-backed skills were no longer executing in those test apps because the harnesses never populated `TimelineLibrary<String>` or registered blueprint-owned extensions. Updating `tests/follow_up_triggers.rs`, `tests/follow_up_chains.rs`, and `tests/pipeline_dispatch.rs` to compile and install the canonical timeline library restored same-update lifecycle and follow-up behavior in the test environment. With those harness fixes in place, the fresh runtime closeout battery passed again: `cargo test`, `cargo check`, and `cargo check --features windowed` all exited 0 on the integrated tree. T02 then remained the artifact bridge that maps this fresh evidence back to milestone validation. The slice closes truthfully by separating runtime proof from architecture-boundary audits: the shared-name grep and blueprint Bevy-import grep still report matches, so S15 records them explicitly as unresolved limitations instead of claiming every original grep gate is already green.

## Verification

Fresh slice-level verification in this message:
- `cargo test` exited 0 after fixing the boot-validation and timeline-harness regressions.
- `cargo check` exited 0.
- `cargo check --features windowed` exited 0.
- The `enum Effect` audit reports no matches.
- The shared-name audit and blueprint Bevy-import audit still return matches, and those outcomes are recorded as explicit limitations rather than hidden failures.

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

S15 no longer claims that every original architecture grep gate is green. Instead, it closes with green runtime verification plus explicit evidence that shared naming and blueprint Bevy-import boundary work still remains on the current tree.

## Known Limitations

The runtime battery is green, but two architecture-boundary audits still show unresolved work on the current tree: shared Digimon-named surfaces still exist outside blueprint-only modules, and blueprint modules still import Bevy directly. S15 records these as explicit limitations rather than claiming those gates pass.

## Follow-ups

Milestone validation must explicitly account for the still-open shared-name audit hits in `src/combat/` and the direct `use bevy` imports that remain under `src/combat/blueprints/`. Those architecture-boundary issues can either be accepted as current scope limits or scheduled into future remediation work.

## Files Created/Modified

- `tests/compiled_timeline_boot_validation.rs` — Reworked the boot-validation test so panic-at-finish and validation-detail assertions use stable harnesses.
- `tests/follow_up_triggers.rs` — Updated the legacy follow-up trigger harness to compile and install the canonical timeline library with blueprint-owned extensions before firing timeline-backed skills.
- `tests/follow_up_chains.rs` — Updated the chain-depth follow-up harness to use the same compiled timeline/runtime registration path as production.
- `tests/pipeline_dispatch.rs` — Updated the lifecycle pipeline harness so timeline-backed skills execute fully inside the test app instead of stalling at declaration/preapp.
