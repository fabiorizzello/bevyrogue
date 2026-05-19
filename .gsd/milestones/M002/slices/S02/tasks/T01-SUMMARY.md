---
id: T01
parent: S02
milestone: M002
key_files:
  - src/combat/runtime/runner.rs
  - tests/timeline_two_clock_parity.rs
  - src/combat/runtime/runner/tests.rs
key_decisions:
  - Windowed `BeatRunner::run_to_completion()` now returns `StepOutcome::AwaitingCue` at presentation barriers instead of auto-resuming; callers must `resume_cue()` and rerun.
  - `resume_cue()` now only flips `cue_just_resumed` when a cue is actually latched, making redundant calls harmless.
duration: 
verification_result: passed
completed_at: 2026-05-19T20:57:21.748Z
blocker_discovered: false
---

# T01: Made Windowed `BeatRunner::run_to_completion()` stop at cue barriers and added parity tests for manual resume without duplicate intents.

**Made Windowed `BeatRunner::run_to_completion()` stop at cue barriers and added parity tests for manual resume without duplicate intents.**

## What Happened

Updated the combat runner’s two-clock contract so batch execution no longer masks presentation barriers in Windowed mode. In `src/combat/runtime/runner.rs`, `StepOutcome::AwaitingCue` docs now describe the batch handshake, `run_to_completion()` returns `AwaitingCue` immediately for `Clock::Windowed`, and `resume_cue()` became a guarded no-op when no cue is latched so stray resumes cannot skip a beat. Reworked `tests/timeline_two_clock_parity.rs` into an explicit handshake fixture with two presentation-bearing beats: HeadlessAuto drains to `Done`, Windowed stops at each barrier, repeated calls without `resume_cue()` prove the cursor stays pinned and no duplicate intents are emitted, and the final normalized `Intent` stream matches HeadlessAuto exactly. I also updated the inline runner unit test in `src/combat/runtime/runner/tests.rs` to replace the old auto-resume assumption with the new resume-and-rerun contract.

## Verification

Verified the required integration test with `cargo test --test timeline_two_clock_parity`, which now runs two assertions: manual Windowed cue handshakes preserve final `Intent` parity with HeadlessAuto, and `resume_cue()` is harmless when no beat is awaiting. Also ran the updated inline runner unit test `cargo test headless_and_windowed_manual_resume_produce_identical_pending_stream --lib` to confirm the library-level contract no longer expects Windowed batch auto-resume.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline_two_clock_parity` | 0 | ✅ pass | 150ms |
| 2 | `cargo test headless_and_windowed_manual_resume_produce_identical_pending_stream --lib` | 0 | ✅ pass | 6853ms |

## Deviations

Also updated `src/combat/runtime/runner/tests.rs` so the inline unit test contract matches the new Windowed `run_to_completion()` behavior; the task plan only named `runner.rs` and `tests/timeline_two_clock_parity.rs` as expected outputs.

## Known Issues

None.

## Files Created/Modified

- `src/combat/runtime/runner.rs`
- `tests/timeline_two_clock_parity.rs`
- `src/combat/runtime/runner/tests.rs`
