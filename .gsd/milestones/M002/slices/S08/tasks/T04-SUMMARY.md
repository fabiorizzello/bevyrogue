---
id: T04
parent: S08
milestone: M002
key_files:
  - tests/timeline/r013_failure_visibility.rs
key_decisions:
  - Initialized `ActionLog` explicitly in the dead-target timeline fixture because `minimal_intent_app()` does not provide the durable observability surface by default.
  - Strengthened the R013 regression to assert overshoot inspectability from the event stream itself: same cast/source/target continuity, damage events continuing after the first `UnitDied`, and a KO→later-hit signature in the capped `ActionLog` tail.
duration: 
verification_result: passed
completed_at: 2026-05-21T22:02:16.855Z
blocker_discovered: false
---

# T04: Hardened the dead-target R013 regression to prove same-cast post-death overshoot is visible in both the combat event stream and ActionLog.

**Hardened the dead-target R013 regression to prove same-cast post-death overshoot is visible in both the combat event stream and ActionLog.**

## What Happened

Hardened the R013 dead-target-mid-loop regression in `tests/timeline/r013_failure_visibility.rs` so it now proves the runtime/presentation flow does not branch on target liveness. The test now initializes `ActionLog`, filters the emitted combat stream down to `OnDamageDealt`/`UnitDied`, asserts all relevant events remain on the same cast against the same target, verifies at least one damage event appears after the first `UnitDied`, and checks the durable `ActionLog` tail retains a KO entry followed by a later hit entry for the same target. While tightening the test, the new assertion surfaced that `minimal_intent_app()` lacks `ActionLog`; the fixture was updated locally in the test to opt into that observability surface explicitly.

## Verification

Ran the focused `r013_failure_visibility` timeline module to confirm the strengthened dead-target assertions and missing `ActionLog` fixture fix, then ran the slice verification command `cargo test --features windowed --test animation --test timeline --test windowed_only`. Both commands passed; the full sweep covered the R009 animation harness, the timeline harness including the hardened R013 regression, and the previously affected windowed-only suites without regressions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline r013_failure_visibility -- --nocapture` | 0 | ✅ pass | 586ms |
| 2 | `cargo test --features windowed --test animation --test timeline --test windowed_only` | 0 | ✅ pass | 10077ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/timeline/r013_failure_visibility.rs`
