---
id: T02
parent: S08
milestone: M002
key_files:
  - src/combat/runtime/cue_barrier.rs
  - src/combat/runtime/mod.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - tests/timeline/r013_failure_visibility.rs
  - tests/timeline/timeline_cue_barrier_pipeline.rs
  - tests/windowed_only/windowed_preview_cache.rs
key_decisions:
  - Added a bounded 180-frame cue-barrier timeout inside `SuspendedTimelineState` and resume it through the same released-runner path as manual cue releases so windowed recovery stays behaviorally aligned with normal completion.
  - Persisted timeout observability on `CueBarrierStatus`/`last_message` with `timed_out`, waited/timeout frame counts, and post-timeout outcome text so stalled presentation barriers remain inspectable after recovery clears the active suspension.
  - Skipped the timeout tick on the same frame a barrier is first suspended, so `waited_frames=0` accurately means no idle frame has elapsed yet and the budget counts only subsequent update frames.
duration: 
verification_result: mixed
completed_at: 2026-05-21T21:50:16.192Z
blocker_discovered: false
---

# T02: Added a bounded windowed cue-barrier timeout that force-resumes stalled timelines and leaves structured timeout diagnostics behind.

**Added a bounded windowed cue-barrier timeout that force-resumes stalled timelines and leaves structured timeout diagnostics behind.**

## What Happened

Implemented bounded timeout recovery for windowed timeline cue barriers. `src/combat/runtime/cue_barrier.rs` now defines a 180-frame budget, extends `CueBarrierStatus` with `timed_out`, `waited_frames`, and `timeout_frames`, adds an internal `TimedOut` release result, and preserves structured timeout diagnostics in `last_status`/`last_message` with cast, skill, timeline, beat, cue, hop, and animation context plus the post-timeout outcome. `SuspendedTimeline` now skips counting the frame where suspension is first latched so the budget starts on the next idle update. `src/combat/turn_system/pipeline/timeline_exec.rs` now ticks the timeout before attempting to resume any released barrier, which lets a timed-out windowed action force-resume through the normal runner path without touching headless behavior. Re-exported the timeout constant from `src/combat/runtime/mod.rs` for integration tests. Updated `tests/timeline/r013_failure_visibility.rs` so R013 now proves a never-released cue remains inspectable until the frame budget expires, then times out, force-resumes, and leaves structured timeout diagnostics behind. Added a parallel regression in `tests/timeline/timeline_cue_barrier_pipeline.rs` covering the integrated timeout path. Updated the `CueBarrierStatus` literal in `tests/windowed_only/windowed_preview_cache.rs` for the expanded structured status shape.

## Verification

Verified the task’s required R013 harness and the integrated cue-barrier pipeline harness. After fixing two test-shape issues (broken trailing delimiters and missing runtime constant imports) and one behavioral issue (the timeout counter initially ticked on the same frame as suspension), `cargo test --test timeline r013_failure_visibility` passed with all three R013 cases green, including the new timeout-recovery contract. Then `cargo test --test timeline timeline_cue_barrier_pipeline` passed, including the new windowed timeout regression proving structured diagnostics plus single-resume behavior through the real pipeline.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline r013_failure_visibility` | 101 | ❌ fail | 2505ms |
| 2 | `cargo test --test timeline r013_failure_visibility` | 101 | ❌ fail | 711ms |
| 3 | `cargo test --test timeline r013_failure_visibility` | 101 | ❌ fail | 2283ms |
| 4 | `cargo test --test timeline r013_failure_visibility` | 0 | ✅ pass | 583ms |
| 5 | `cargo test --test timeline timeline_cue_barrier_pipeline` | 0 | ✅ pass | 223ms |

## Deviations

None.

## Known Issues

Unrelated warning remains in `tests/timeline/timeline_loop_hop_cue_parity.rs` for an unused `BeatEdge` import during the timeline test harness build.

## Files Created/Modified

- `src/combat/runtime/cue_barrier.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `tests/timeline/r013_failure_visibility.rs`
- `tests/timeline/timeline_cue_barrier_pipeline.rs`
- `tests/windowed_only/windowed_preview_cache.rs`
