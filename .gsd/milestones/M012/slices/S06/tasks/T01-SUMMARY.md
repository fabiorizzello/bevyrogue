---
id: T01
parent: S06
milestone: M012
key_files:
  - src/combat/action_query.rs
  - tests/action_affordance_query.rs
key_decisions:
  - Reused existing query helpers to ensure consistency across the affordance API.
  - Enforced priority chain by sequential validation: missing skill > implementation > actor/phase/resource > target status.
duration: 
verification_result: passed
completed_at: 2026-05-01T09:46:29.889Z
blocker_discovered: false
---

# T01: Add pure query_intent_legality helper and reason-priority tests

**Add pure query_intent_legality helper and reason-priority tests**

## What Happened

Implemented `query_intent_legality` as a pure function in `src/combat/action_query.rs`. The function resolves the skill, checks implementation status, validates actor/phase/resource state via `action_and_resource_status_for_snapshot`, and finally performs a specific target check via `target_status_for_unit`. Added a comprehensive integration test `intent_legality_respects_priority_and_specific_target_reasons` that covers valid intents, specific target failures (WrongSide, TargetKo, etc.), and the priority of actor-level failures (AttackerKo, SpShortfall) over target-level ones.

## Verification

Ran `cargo test --test action_affordance_query` which confirmed all 23 tests pass, including the new 10+ scenario priority test.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test action_affordance_query` | 0 | ✅ pass | 3996ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
