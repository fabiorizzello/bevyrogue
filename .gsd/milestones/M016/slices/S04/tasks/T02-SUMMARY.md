---
id: T02
parent: S04
milestone: M016
key_files:
  - assets/data/skills.ron
  - tests/twin_core_integration.rs
key_decisions:
  - Attached Twin Core custom signals to Agumon and Gabumon skills in skills.ron.
  - Added integration test to verify that skill resolution correctly emits blueprint-driven Twin Core tag transitions.
duration: 
verification_result: passed
completed_at: 2026-05-10T21:54:39.142Z
blocker_discovered: false
---

# T02: Migrate Twin Core skills to custom signals and verify via integration tests.

**Migrate Twin Core skills to custom signals and verify via integration tests.**

## What Happened

I have updated `assets/data/skills.ron` to include `custom_signals` for Agumon and Gabumon's skills (`pepper_breath`, `bubble_blast`, `agumon_ult`, `gabumon_ult`, `agumon_follow_up`, `gabumon_follow_up`). These signals are now routed through the blueprints created in the previous task. 

I also updated `tests/twin_core_integration.rs` to include a new test case, `skill_resolution_emits_twin_core_signals_through_blueprints`, which verifies that resolving Agumon's basic skill (`pepper_breath`) correctly results in a `CombatKernelTransition::Tag` addition for the `Heated` tag, handled via the Agumon blueprint.

During execution, I had to fix some corruption in `skills.ron` caused by partial edits and corrected compilation errors in the integration test related to the `UnitSkills` struct shape and `CombatTagState` field access.

## Verification

Ran `cargo test --test twin_core_integration` and verified that both existing and new tests pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test twin_core_integration` | 0 | ✅ pass | 1000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `assets/data/skills.ron`
- `tests/twin_core_integration.rs`
