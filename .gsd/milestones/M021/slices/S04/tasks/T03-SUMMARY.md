---
id: T03
parent: S04
milestone: M021
key_files:
  - src/combat/api/passive_runner.rs
  - src/combat/api/runner_common.rs
  - src/combat/api/runner.rs
  - src/combat/plugin.rs
key_decisions:
  - Lift fire_beat/next_beat to shared runner_common module to avoid duplication between BeatRunner and PassiveRunner.
  - Use resource_scope in passive_dispatch_system to manage multi-resource mutation while maintaining access to the World for SkillCtx.
duration: 
verification_result: passed
completed_at: 2026-05-15T12:02:48.167Z
blocker_discovered: false
---

# T03: Implemented PassiveRunner and passive_dispatch_system with shared execution helpers and signal-cascade circuit-breaker.

**Implemented PassiveRunner and passive_dispatch_system with shared execution helpers and signal-cascade circuit-breaker.**

## What Happened

Implemented the reactive layer's core execution engine: PassiveRunner. 

Refactored the existing BeatRunner to extract shared execution logic (fire_beat, next_beat, find_beat) into a new src/combat/api/runner_common.rs module. This allows both runners to share the same hook firing, selector resolution, and edge gating logic while maintaining their distinct driving events (cursor advance vs signal arrival).

The passive_dispatch_system was implemented as an exclusive Bevy system that drains the SignalBus and iterates over PassiveListeners. It includes a signal-cascade circuit-breaker (MAX_HOPS=256) to prevent infinite loops where passives trigger further signals. To handle the borrow complexity of mutating multiple resources (PassiveListeners, IntentQueue, CastIdGen) while needing read-only World access for SkillCtx (predicates), a nested resource_scope pattern with unique shadowing was employed.

Verified the implementation with inline unit tests in passive_runner.rs (trigger matching and circuit-breaker) and ensured no regressions in existing timeline tests.

## Verification

Ran inline unit tests in src/combat/api/passive_runner.rs covering trigger predicate matching and circuit-breaker halting. Ran existing integration tests (timeline_mode_parity, timeline_two_clock_parity, timeline_circuit_breaker) to ensure no regressions. All checks passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib -- combat::api::passive_runner` | 0 | ✅ pass | 1690ms |
| 2 | `cargo test --test timeline_circuit_breaker --test timeline_mode_parity --test timeline_two_clock_parity` | 0 | ✅ pass | 1090ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/passive_runner.rs`
- `src/combat/api/runner_common.rs`
- `src/combat/api/runner.rs`
- `src/combat/plugin.rs`
