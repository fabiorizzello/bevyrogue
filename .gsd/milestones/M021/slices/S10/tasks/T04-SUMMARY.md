---
id: T04
parent: S10
milestone: M021
key_files:
  - tests/dorumon_predator_runtime.rs
key_decisions:
  - Treat the Dorumon predator regression as a single canonical transition pass; duplicate blueprint emission is a test artifact, not the owner/runtime contract.
duration: 
verification_result: passed
completed_at: 2026-05-17T06:35:40.820Z
blocker_discovered: false
---

# T04: Removed the duplicate Dorumon predator event injection so the runtime regression now ends on the applied prey-lock transition under the generic blueprint contract.

**Removed the duplicate Dorumon predator event injection so the runtime regression now ends on the applied prey-lock transition under the generic blueprint contract.**

## What Happened

The Dorumon predator runtime failure was caused by the regression test itself enqueueing the same blueprint transition sequence twice before a single update. That duplicated pass pushed the second apply_prey_lock into the existing active prey-lock cap and made the final last_transition a CapReached { cap: PreyLock } rejection, even though the owner/runtime implementation was already producing the correct single-pass transition flow. I simplified the focused regression to a single canonical transition pass, kept the existing state and snapshot assertions, and left the shared kernel/event implementation unchanged because it was already matching the intended contract.

## Verification

Ran cargo test --test dorumon_predator_runtime --quiet and cargo test --test event_stream --quiet after the regression cleanup; both passed. The Dorumon runtime test now confirms the canonical blueprint transitions produce exploit stacks 2, an active prey-lock with the expected duration, and last_transition = applied prey-lock rather than a cap rejection.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test dorumon_predator_runtime --quiet` | 0 | ✅ pass | 317ms |
| 2 | `cargo test --test event_stream --quiet` | 0 | ✅ pass | 337ms |

## Deviations

No source/runtime code changes were required; the correction was entirely in the focused regression coverage.

## Known Issues

None.

## Files Created/Modified

- `tests/dorumon_predator_runtime.rs`
