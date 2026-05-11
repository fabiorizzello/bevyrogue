---
id: T04
parent: S05
milestone: M011
key_files:
  - tests/resource_caps.rs
key_decisions:
  - sp_non_basic_cap_enforced tests RoundSpTracker API directly (not through Bevy pipeline) because _sp_tracker in apply_effects is a local-per-call variable and is not persisted as a Bevy Resource — the cap logic lives in try_gain_non_basic, not in the resolution pipeline
  - child_discount_after_two_basics uses the full resolve_action_system pipeline so it exercises the complete BasicStreak mutation path through ResolveActorsQuery element 11
duration: 
verification_result: passed
completed_at: 2026-04-27T20:23:44.513Z
blocker_discovered: false
---

# T04: Added integration tests for Child SP discount scenario (end-to-end via Bevy pipeline) and SP non-basic cap (RoundSpTracker direct API)

**Added integration tests for Child SP discount scenario (end-to-end via Bevy pipeline) and SP non-basic cap (RoundSpTracker direct API)**

## What Happened

Created `tests/resource_caps.rs` with two integration tests proving the S05 slice contract.

`child_discount_after_two_basics` runs through the full Bevy `resolve_action_system` pipeline. A Child unit is spawned with `BasicStreak::default()` and `EvoStage::Child`. After two Basic actions the streak counter reaches 2 and SpPool is at 5. A Skill action (cost 3) fires the discount: the pipeline reads the `BasicStreak` component from the ECS query (element 11 of `ResolveActorsQuery`) and passes a `&mut BasicStreak` to `apply_effects`, which applies a -1 discount and resets the streak. The test asserts SP drops from 5 to 3 (not 2), and the streak component reads 0 after the call. A second immediate Skill (streak = 0) asserts SP drops from 3 to 0, confirming no discount on a fresh streak.

`sp_non_basic_cap_enforced` tests the `RoundSpTracker` API directly (no Bevy App). This is intentional: per the T01 decision, `_sp_tracker` is a local-default variable in `pipeline::step_app` and is passed to `apply_effects` but not wired into any gain logic — the cap is enforced by the caller choosing how many SP to grant, not by the pipeline itself. The test calls `try_gain_non_basic(1)` three times, asserts only 2 are granted, then resets and asserts 2 more succeed.

## Verification

cargo test --test resource_caps: 2/2 pass (child_discount_after_two_basics, sp_non_basic_cap_enforced). cargo test (full suite): 0 failures across all test binaries.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test resource_caps 2>&1 | tail -10` | 0 | ✅ pass — 2 tests passed, 0 failed | 550ms |
| 2 | `cargo test 2>&1 | grep -E 'test result|FAILED|error'` | 0 | ✅ pass — all test binaries report 0 failures | 2000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/resource_caps.rs`
