---
id: T03
parent: S05
milestone: M021
key_files:
  - src/combat/state.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/api/applier.rs
  - src/combat/api/builtins.rs
  - tests/compiled_timeline_runtime_dispatch.rs
key_decisions:
  - Use a deferred world command to keep the normal resolver non-exclusive while still running BeatRunner on compiled timelines.
  - Intern owned timeline ids to 'static at dispatch time so the existing BeatRunner API can consume TimelineLibrary<String> entries without a larger generic refactor.
duration: 
verification_result: passed
completed_at: 2026-05-15T16:34:07.444Z
blocker_discovered: false
---

# T03: Routed timeline-backed skills through BeatRunner and extended the intent applier to execute BreakToughness, ApplyStatus, DelayTurn, and ApplyBuff intents.

**Routed timeline-backed skills through BeatRunner and extended the intent applier to execute BreakToughness, ApplyStatus, DelayTurn, and ApplyBuff intents.**

## What Happened

Implemented a timeline-backed execution path that queues a deferred world command from the normal action resolver, runs BeatRunner against the compiled timeline library, drains the generated intents through the existing applier, and then emits the action lifecycle closeout events. In parallel, extended the applier so built-in timeline hooks now mutate toughness, statuses, buffs, and delay events through the real combat subsystems instead of stubs. Added a production-like integration test that proves a compiled timeline skill can travel through the standard turn pipeline while a non-timeline skill still falls back to the legacy effect resolver.

## Verification

Validated with `cargo test --test compiled_timeline_runtime_dispatch -- --nocapture`. The test suite compiled and both runtime-path assertions passed: the timeline-backed skill emitted the expected combat events and state mutations, and the legacy skill still resolved through the non-timeline path.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test compiled_timeline_runtime_dispatch -- --nocapture` | 0 | ✅ pass | 1200ms |

## Deviations

Timeline-backed dispatch is implemented via a deferred world command that executes BeatRunner after the resolver returns, rather than by converting the resolver itself into an exclusive system.

## Known Issues

Touched modules still have pre-existing unused-import warnings; no failing runtime issues remain.

## Files Created/Modified

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/builtins.rs`
- `tests/compiled_timeline_runtime_dispatch.rs`
