---
id: T02
parent: S06
milestone: M012
key_files:
  - src/combat/action_query.rs
  - src/combat/turn_system/mod.rs
  - tests/revive_semantics.rs
  - tests/target_shape_truthfulness.rs
key_decisions:
  - Used `as_readonly()` on Bevy queries to safely build the immutable `CombatQuerySnapshot` without violating mutable borrow rules for downstream pipeline steps.
  - Switched SP bypass value to `i32::MAX` after discovering that `-1` (from `u32::MAX as i32`) triggered shortfall rejections in the legality helper.
  - Updated integration tests to assert on `Debug`-formatted enum variants (e.g., 'TargetNotKo') to align with the new machine-readable verification contract.
duration: 
verification_result: passed
completed_at: 2026-05-01T10:46:58.220Z
blocker_discovered: false
---

# T02: Build ECS snapshot adapter and wire early validation into resolve_action_system

**Build ECS snapshot adapter and wire early validation into resolve_action_system**

## What Happened

Implemented `build_snapshot_from_ecs` in `src/combat/action_query.rs` to bridge Bevy state into the pure validation layer. Integrated this as an early guard in `resolve_action_system`, ensuring that illegal actions are rejected before any declaration or mutation occurs. Handled a significant lifetime constraint by leveraging Bevy's `as_readonly()` query conversion, which allowed collecting unit data into a transient `Vec` without locking the mutable `actors` query needed by the execution pipeline. Verified that SP validation is correctly bypassed in this early stage to preserve existing lifecycle event contracts for SP failures. Corrected several integration tests that were still expecting legacy string-based failure reasons.

## Verification

Ran integration tests for target shape truthfulness, revive semantics, and pipeline dispatch. All tests passed, confirming that early validation correctly rejects illegal intents (like attacking KO targets or using unimplemented shapes) while allowing valid ones to proceed through the full lifecycle. verified that SP shortfall still follows the legacy lifecycle path as required.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test target_shape_truthfulness --test revive_semantics --test pipeline_dispatch --test action_affordance_query` | 0 | ✅ pass | 4890ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/action_query.rs`
- `src/combat/turn_system/mod.rs`
- `tests/revive_semantics.rs`
- `tests/target_shape_truthfulness.rs`
