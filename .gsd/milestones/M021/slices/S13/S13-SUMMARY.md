---
id: S13
parent: M021
milestone: M021
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions: []
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-17T13:34:56.685Z
blocker_discovered: false
---

# S13: Close deferred foundation captures and boot invariants

**Captured fresh proof for cast_id propagation, UltInstant/turn-pipeline ordering, DryRun-vs-Execute parity, and strict boot-time validation of invalid timeline ids.**

## What Happened

T03 extended the compiled timeline boot-validation regression surface with a focused App::finish() harness that injects a dangling timeline reference and proves CombatPlugin rejects invalid hook and predicate ids with aggregated panic output. The existing T01 and T02 proof surfaces remain in place for cast_id, UltInstant, turn-phase ordering, and DryRun parity, so the slice now closes the deferred M021 foundation gap with fresh live evidence instead of historical roadmap claims. The task summary artifacts record the implementation details and the exact panic text observed from the boot failure path.

## Verification

Verified the focused boot-validation regression by running `cargo test --test compiled_timeline_boot_validation -- --nocapture invalid_timeline_ids_fail_during_app_finish`, which deterministically failed at the intended boot seam and emitted the aggregated dangling-reference panic text containing both the missing hook and missing predicate. Also verified the regression and boot seam are present in the test file and companion plugin/timeline surfaces via targeted inspection, while T01/T02 task summaries document the cast_id, UltInstant, turn-phase, and DryRun parity evidence already captured for the slice.

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

None.

## Follow-ups

None.

## Files Created/Modified

None.
