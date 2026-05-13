---
id: T02
parent: S02
milestone: M017
key_files:
  - src/combat/bootstrap.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - tests/combat_coherence.rs
  - tests/status_accuracy.rs
  - tests/status_effect_apply.rs
key_decisions:
  - Added a fallback branch in pipeline.rs apply site to insert a fresh StatusBag when the entity lacks one — guards test fixtures spawned without the component while still working correctly post-bootstrap-seeding.
  - Updated three test files to use StatusBag instead of StatusEffect for query and assertion, rather than keeping StatusEffect alive via the deprecated shim.
  - Named the defender tuple element 10 as mut defender_bag in the get_many_mut destructure instead of adding a separate Query, avoiding borrow conflicts.
duration: 
verification_result: passed
completed_at: 2026-05-13T07:03:30.316Z
blocker_discovered: false
---

# T02: Migrated apply pipeline from StatusEffect to StatusBag; all units now seeded with StatusBag::default() at spawn; tests updated.

**Migrated apply pipeline from StatusEffect to StatusBag; all units now seeded with StatusBag::default() at spawn; tests updated.**

## What Happened

Bootstrap seeding: added `StatusBag::default()` to the `spawn_unit_from_def` bundle in `src/combat/bootstrap.rs`, adjacent to `RoundFlags::default()`. Also imported `super::status_effect::StatusBag` in that file.

Pipeline migration: in `src/combat/turn_system/pipeline.rs`, replaced the `use crate::combat::status_effect::StatusEffect` import with `StatusBag`. Named the previously-unnamed `_` element 10 in the defender destructure as `mut defender_bag`. Replaced the `commands.entity(target_entity).insert(StatusEffect { kind, duration_remaining })` call with a query-driven `bag.apply(kind.clone(), duration)` call. A fallback path using `commands.entity(target_entity).insert(fresh_bag)` guards against units spawned without `StatusBag` (e.g. old test fixtures). `OnStatusApplied` is emitted unconditionally after `apply()` returns (covers both insert and refresh). `OnStatusResisted` remains gated by `roll_pct(threshold)` as before.

`ResolveActorsQuery` in `src/combat/turn_system/mod.rs` was already updated by the linter to use `StatusBag` at index 10, and `advance_turn_system` was also updated to use `StatusBag` with `tick_all()` + per-kind `OnStatusExpired` events.

Test updates: three test files (`tests/combat_coherence.rs`, `tests/status_accuracy.rs`, `tests/status_effect_apply.rs`) were updated to: (1) import `StatusBag` instead of `StatusEffect`; (2) add `StatusBag::default()` to all manually-spawned unit bundles; (3) replace `StatusEffect` component presence assertions with `StatusBag::has(kind)` / `StatusBag::is_empty()` checks. The `reapply` test was updated to pre-populate a `StatusBag` with `apply(Heated, 1)` instead of inserting a `StatusEffect` struct, and asserts `max(1, 3) = 3` per refresh_max_dur policy.

## Verification

cargo check clean (no errors, 14 warnings in lib — all pre-existing). cargo test: all test suites pass (0 failures). Verified: StatusBag::default() present in bootstrap.rs; StatusEffect import absent from pipeline.rs; OnStatusApplied fires after apply(); OnStatusResisted still gated by roll_pct; rg confirms ≥1 match for StatusBag::default in bootstrap.rs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check 2>&1 | grep -E '^error|Finished'` | 0 | PASS — no compilation errors | 150ms |
| 2 | `cargo test 2>&1 | grep 'test result:'` | 0 | PASS — all test suites green, 0 failures | 2100ms |
| 3 | `grep -n 'StatusBag::default' src/combat/bootstrap.rs` | 0 | PASS — StatusBag::default() present at line 162 | 10ms |
| 4 | `grep -n 'StatusEffect' src/combat/turn_system/pipeline.rs` | 1 | PASS — no StatusEffect references in pipeline.rs (exit 1 = no matches) | 10ms |

## Deviations

Minor scope expansion: updated three integration test files (combat_coherence.rs, status_accuracy.rs, status_effect_apply.rs) to compile and pass against the new StatusBag API. These were not listed in Expected Output but were required to keep the test suite green.

## Known Issues

None.

## Files Created/Modified

- `src/combat/bootstrap.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/combat_coherence.rs`
- `tests/status_accuracy.rs`
- `tests/status_effect_apply.rs`
