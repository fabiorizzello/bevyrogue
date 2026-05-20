---
id: T04
parent: S02
milestone: M002
key_files:
  - src/combat/runtime/cue_barrier.rs
  - src/combat/runtime/runner.rs
  - src/combat/runtime/mod.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - src/combat/turn_system/pipeline/mod.rs
  - src/combat/turn_system/resolve.rs
  - src/combat/turn_system/mod.rs
  - tests/timeline_cue_barrier_pipeline.rs
key_decisions:
  - Persist cue-gated actions in `SuspendedTimelineState` with the live `BeatRunner`, buffered intents, and inflight metadata so resume reuses the same runner instead of rebuilding execution state.
  - Keep `CombatPhase::Resolving` latched while a cue barrier is awaiting release, and drain any incoming action intents during that phase so windowed spam cannot accumulate additional suspended runners.
  - Route both fresh completion and resumed completion through one shared timeline-finalization path so SP spend, ult bookkeeping, intent application, and action events stay deterministic across clocks.
duration: 
verification_result: passed
completed_at: 2026-05-20T07:09:35.148Z
blocker_discovered: false
---

# T04: Persisted cue-gated timeline actions across frames and only commit their intents after an explicit cue release resumes the suspended runner.

**Persisted cue-gated timeline actions across frames and only commit their intents after an explicit cue release resumes the suspended runner.**

## What Happened

Added a generic cue-barrier runtime seam for timeline-backed actions. `TimelineClock` now selects whether new timeline casts run headlessly or suspend at presentation beats, and `SuspendedTimelineState` persists the in-flight `BeatRunner`, buffered pending intents, cast metadata, and inspectable barrier diagnostics (cast_id, skill_id, timeline_id, beat_id, cue_id, awaiting/released state, optional animation node/frame placeholders). `run_timeline_backed_action` now sets `CombatPhase::Resolving`, drives the runner with the selected clock, stores suspended executions on `AwaitingCue`, and only commits pending intents after a later resume reaches `Done`. The old tail work in `timeline_exec.rs` was factored into a shared finalization path used by both initial and resumed completion, while failure/preflight paths still emit the expected action-failure signals and reset the phase. A public `continue_suspended_timeline_system` plus `request_timeline_cue_release()` API were exposed for future windowed animation code, with duplicate/no-suspended releases logged as explicit no-op diagnostics. `resolve_action_system` now drains action intents while the phase is `Resolving`, preventing spam from piling up extra suspended runners. Finally, `tests/timeline_cue_barrier_pipeline.rs` exercises the positive parity path and the negative/load cases: release-before-suspension, duplicate release, spam while resolving, halted resume, and no duplicate damage after completion.

## Verification

Verified the new barrier pipeline end to end with the task command `cargo test --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity --test compiled_timeline_runtime_dispatch`, which passed and covered headless/windowed parity, the new suspended-runner pipeline, duplicate/no-op release handling, halted resume failure cleanup, and existing compiled timeline dispatch behavior. Ran `cargo test --test timeline_cue_barrier_pipeline` again for focused confirmation of the new integration cases; all 5 tests passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cd /home/fabio/dev/bevyrogue && cargo test --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity --test compiled_timeline_runtime_dispatch` | 0 | ✅ pass | 574ms |
| 2 | `cd /home/fabio/dev/bevyrogue && cargo test --test timeline_cue_barrier_pipeline` | 0 | ✅ pass | 154ms |

## Deviations

Added a public re-export in `src/combat/turn_system/mod.rs` so integration tests and future windowed code can schedule the continuation system without reaching into the private `pipeline` module; otherwise the implementation followed the task plan.

## Known Issues

None.

## Files Created/Modified

- `src/combat/runtime/cue_barrier.rs`
- `src/combat/runtime/runner.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/combat/turn_system/pipeline/mod.rs`
- `src/combat/turn_system/resolve.rs`
- `src/combat/turn_system/mod.rs`
- `tests/timeline_cue_barrier_pipeline.rs`
