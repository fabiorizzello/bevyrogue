---
id: T03
parent: S03
milestone: M021
key_files:
  - tests/timeline_two_clock_parity.rs
key_decisions:
  - Used CastId::ROOT as the cast_id (deterministic, no CastIdGen resource needed for a runner-only test)
  - Encoded beat identity in DealDamage amount (7 for cast, 13 for impact) so the intent stream is distinguishable and deterministic without world state
  - Manual Windowed loop tracks awaiting_cue_count and last stall beat to assert stall is real and located at the presentation beat, satisfying both the 'stall observed' and 'stream parity' invariants in a single test function
duration: 
verification_result: passed
completed_at: 2026-05-15T08:43:15.622Z
blocker_discovered: false
---

# T03: Added integration test asserting HeadlessAuto and Windowed clocks produce identical end-of-cast Intent streams, with proof that the Windowed stall is real (AwaitingCue observed and resume_cue called).

**Added integration test asserting HeadlessAuto and Windowed clocks produce identical end-of-cast Intent streams, with proof that the Windowed stall is real (AwaitingCue observed and resume_cue called).**

## What Happened

Created tests/timeline_two_clock_parity.rs from scratch. The test builds a two-beat timeline (Cast with Presentation → Impact without) and a shared hook that enqueues a DealDamage intent per beat (amounts 7 and 13 respectively, deterministic). Run #1 uses HeadlessAuto via run_to_completion. Run #2 uses Clock::Windowed via a manual step() loop bounded at 64 iterations: on AwaitingCue it increments a counter, records the beat id as "cast", calls resume_cue(), then continues; stops on Done/Halted. Final assertions: (a) windowed_outcome == Done, (b) awaiting_cue_count >= 1 proving the stall is real not bypassed, (c) last_awaiting_cue_beat == Some("cast") proving the stall location, (d) format!("{:?}") of every intent in both pending queues is equal (stream parity, I3/D026). Removed an unused CastIdGen import caught by the compiler. Test compiled and passed clean in 0.00s with no warnings.

## Verification

cargo test --test timeline_two_clock_parity 2>&1 | tail -10 — 1 passed, 0 failed, no warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test timeline_two_clock_parity 2>&1 | tail -10` | 0 | PASS — 1 test passed, 0 failed, no warnings | 410ms |

## Deviations

None. The inline unit tests in runner.rs (tests f: headless_and_windowed_produce_identical_pending_stream) already cover the same invariant in unit-test form; this integration test adds coverage via the public crate surface and the manual-step Windowed drive pattern specified by the task plan.

## Known Issues

None.

## Files Created/Modified

- `tests/timeline_two_clock_parity.rs`
