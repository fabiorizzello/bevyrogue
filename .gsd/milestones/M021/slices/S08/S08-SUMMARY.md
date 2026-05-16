---
id: S08
parent: M021
milestone: M021
provides:
  - Validated Twin Core extraction and Bouncing Fire gating for downstream migration slices.
  - A deterministic regression harness for the shared Twin Core blueprint path.
requires:
  []
affects:
  - S09
  - S10
key_files:
  - tests/bouncing_fire_off_baseline.rs
  - .gsd/milestones/M021/slices/S08/S08-PLAN.md
  - .gsd/milestones/M021/slices/S08/tasks/T04-SUMMARY.md
  - .gsd/milestones/M021/slices/S08/tasks/T04-VERIFY.json
key_decisions:
  - Move Twin Core onto the shared Blueprint owner path instead of a kernel-local TwinCore transition variant.
  - Use a gate-only Bouncing Fire loop branch so talent rank 0 preserves baseline intent output exactly.
  - Keep Gabumon as a directory module and consume Twin Core from the shared module namespace.
patterns_established:
  - Shared blueprint transitions should flow through the generic Blueprint owner payload instead of bespoke kernel variants.
  - Deterministic regression tests should pin target identity / event shape rather than fragile post-mitigation numbers when possible.
  - A zero-rank talent gate should be implemented so OFF == baseline by construction.
observability_surfaces:
  - Blueprint owner transition events in the combat JSONL stream.
  - Deterministic regression tests for OFF baseline and Twin Core migration.
drill_down_paths:
  - .gsd/milestones/M021/slices/S08/tasks/T04-SUMMARY.md
  - .gsd/milestones/M021/slices/S08/tasks/T04-VERIFY.json
duration: ""
verification_result: passed
completed_at: 2026-05-16T22:15:54.132Z
blocker_discovered: false
---

# S08: S08: Agumon + Gabumon migrated (Twin Core paired)

**Twin Core now flows through the shared blueprint path, Gabumon imports the extracted twin_core module, and Bouncing Fire proves OFF=baseline with deterministic tests.**

## What Happened

T01 removed the old TwinCore coupling from kernel-local code and consolidated the shared transition path under the blueprint surface. T02 converted Gabumon into a directory module and switched its TwinCore imports to the shared twin_core module. T03 added the Bouncing Fire loop branch to baby_flame with the talent predicate, selector, and hook wiring, plus the supporting timeline/registry changes needed for a gated loop branch. T04 added deterministic end-to-end coverage in tests/bouncing_fire_off_baseline.rs for both the OFF baseline and rank-1 bounce path and verified the migrated Twin Core path remains green through the Blueprint event stream.

Fresh verification in this session confirmed the slice-specific checks passed: cargo test --test bouncing_fire_off_baseline and cargo test twin_core both exited 0. A broader cargo test run still exposes unrelated pre-existing failures in tests/follow_up_triggers.rs and tests/combat_coherence.rs; those were already outside this slice’s implementation scope and were not introduced by S08.

## Verification

Verified in-session with gsd_exec: cargo test --test bouncing_fire_off_baseline ✅, cargo test twin_core ✅. Broader suite checks still report unrelated pre-existing failures outside this slice (follow_up_triggers.rs and combat_coherence.rs), which were documented as known limitations rather than slice regressions.

## Requirements Advanced

- M021 success criteria — validated the Twin Core migration path and Bouncing Fire OFF baseline at the slice level.

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

Broader repository verification still has unrelated pre-existing failures in tests/follow_up_triggers.rs and tests/combat_coherence.rs; S08 does not address those.

## Follow-ups

None.

## Files Created/Modified

- `tests/bouncing_fire_off_baseline.rs` — Added deterministic OFF-baseline and rank-1 Bouncing Fire regression coverage.
- `.gsd/milestones/M021/slices/S08/S08-PLAN.md` — Marked S08 tasks complete in the slice plan.
- `.gsd/milestones/M021/slices/S08/tasks/T04-SUMMARY.md` — Recorded the task-level summary for the new regression tests.
- `.gsd/milestones/M021/slices/S08/tasks/T04-VERIFY.json` — Recorded verification evidence for the slice task.
