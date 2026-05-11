---
id: T03
parent: S04
milestone: M012
key_files:
  - src/combat/action_query.rs
  - tests/action_affordance_query.rs
key_decisions:
  - ActionQueryKind now resolves via `SkillId` for skill actions, while Basic and Ultimate are looked up from the actor's `UnitSkills` and the skill book.
  - Action/resource failures stay reason-bearing, but non-missing results always include the per-target affordance surface so callers can inspect both action and target legality in one call.
  - Resource affordances expose current/required values for SP and ultimate readiness through a dedicated detail list rather than overloading the coarse status enums.
duration: 
verification_result: passed
completed_at: 2026-05-01T07:05:16.194Z
blocker_discovered: false
---

# T03: Implemented a pure action affordance query with stable action/resource/target reasons and resource detail output.

**Implemented a pure action affordance query with stable action/resource/target reasons and resource detail output.**

## What Happened

Added `query_action_affordance` to `src/combat/action_query.rs` so callers can resolve Basic, Skill, and Ultimate actions through the actor's kit plus the DSL-backed skill book in one pure call. The query now enforces the requested failure priority, returns structured action/resource/implementation statuses, carries per-target affordances even when the actor state blocks execution, and exposes SP plus ultimate readiness details with current/required values. I also expanded the test surface in `tests/action_affordance_query.rs` to pin the happy path and the required negative cases: non-active actors, wrong phase, KO/stunned attackers, missing skill data, SP shortfall, ultimate not ready, no valid targets, and hidden/deferred implementations.

## Verification

Ran `cargo test-dev --test action_affordance_query` after the final code changes. The targeted suite passed with 17/17 tests green, confirming the pure action query, target affordances, resource details, and stable reason codes behave as expected.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_query` | 0 | ✅ pass | 215ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
