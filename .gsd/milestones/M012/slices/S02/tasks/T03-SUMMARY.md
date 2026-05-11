---
id: T03
parent: S02
milestone: M012
key_files:
  - src/combat/state.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - src/combat/follow_up.rs
  - src/combat/resolution_tests.rs
  - tests/target_shape_truthfulness.rs
  - tests/follow_up_triggers.rs
key_decisions:
  - Carry `TargetShape` on `ResolvedAction` so legality can be decided after DSL resolution but before mutation.
  - Reject non-single shapes in `step_declaration` with a stable `UnimplementedTargetShape:<Shape>` failure and no lifecycle side effects.
  - Keep row-shaped canonical follow-up fixtures from masquerading as single-target by switching them to single-shape skills.
duration: 
verification_result: passed
completed_at: 2026-04-30T21:30:18.675Z
blocker_discovered: false
---

# T03: Preserved target-shape metadata and rejected unsupported Row/AllEnemies actions before mutation

**Preserved target-shape metadata and rejected unsupported Row/AllEnemies actions before mutation**

## What Happened

I reproduced the shape-truthfulness gap with a failing integration test, then wired `ResolvedAction` to retain `TargetShape` metadata from the first damage effect and default no-damage skills to `Single`. I added a shared `target_shape_is_executable_now`/rejection helper and taught `step_declaration` to log and emit `OnActionFailed` with a stable `UnimplementedTargetShape:<Shape>` reason before any declaration or mutation lifecycle events. I also updated the follow-up fixture set so canonical Row skills are no longer used as ordinary single-target stand-ins, and I pinned the new contract with unit tests for preserved metadata plus integration tests for Row, AllEnemies, and Single behavior.

## Verification

Verified the rejection and lifecycle contract with `cargo test-dev --test target_shape_truthfulness` (3/3 passed), `cargo test-dev --test target_shape_truthfulness --test follow_up_triggers --test pipeline_dispatch` (12/12 passed across the three targets), and a full `cargo test-dev` run (all lib/main/integration/doc tests passed; 124 lib tests + 125 main tests + all integration suites green). The rejected-shape path now surfaces `OnActionFailed`/`ActionLog` evidence and leaves SP/HP/toughness/lifecycle untouched, while Single-shaped skills still execute normally.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test target_shape_truthfulness` | 0 | ✅ pass | 4800ms |
| 2 | `cargo test-dev --test target_shape_truthfulness --test follow_up_triggers --test pipeline_dispatch` | 0 | ✅ pass | 7300ms |
| 3 | `cargo test-dev` | 0 | ✅ pass | 3300ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `src/combat/resolution_tests.rs`
- `tests/target_shape_truthfulness.rs`
- `tests/follow_up_triggers.rs`
