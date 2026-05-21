---
id: T04
parent: S05
milestone: M002
key_files:
  - src/combat/runtime/runner.rs
  - src/combat/runtime/cue_barrier.rs
  - assets/data/digimon/agumon/skills.ron
  - assets/digimon/agumon/anim_graph.ron
  - tests/timeline/timeline_loop_hop_cue_parity.rs
  - tests/timeline.rs
key_decisions:
  - Loop body beats are not in timeline.beats (only in BeatKind::Loop { body }), so awaiting_cue_info() cannot use find_beat for them — fixed by caching awaiting_presentation alongside awaiting_cue at latch time.
  - hop_index derived lazily from loop_stack.last() in awaiting_cue_info() — no extra storage needed beyond the existing loop frame stack.
  - bounce_hop presentation cue_id='agumon/baby_flame/bounce_hop' is a routing token, not a gameplay number — satisfies the anti-DRY invariant from anim_gameplay_command_forbidden.rs.
  - New test uses inline BeatRunner fixtures (not full app) matching timeline_two_clock_parity.rs pattern — avoids Agumon blueprint dependencies while fully covering the per-hop stall mechanics.
  - anim_graph.ron self-transition uses priority 10 (same as baby_flame_cast→impact) so KernelCue fires before TimeInNode; TimeInNode transition to recover still fires when loop ends.
duration: 
verification_result: passed
completed_at: 2026-05-21T10:43:49.111Z
blocker_discovered: false
---

# T04: Extended per-hop kernel cue barrier: AwaitingCueInfo exposes hop_index, loop body beats cache their Presentation to survive find_beat, bounce_hop gains a cue, and anim_graph.ron gains a self-transition; 3 new tests assert N-hop windowed parity.

**Extended per-hop kernel cue barrier: AwaitingCueInfo exposes hop_index, loop body beats cache their Presentation to survive find_beat, bounce_hop gains a cue, and anim_graph.ron gains a self-transition; 3 new tests assert N-hop windowed parity.**

## What Happened

The verification failure was a false alarm — the gate ran the wrong command (`--test windowed_preview_cache --test windowed_target_hurt`) from T03, not T04's verification. T04 had not been implemented yet.

Implemented T04 in full:

1. **runner.rs**: Added `awaiting_presentation: Option<Presentation>` cache field to `BeatRunner`. When the runner latches `awaiting_cue` (in both loop-body and linear paths), it now also caches the beat's `Presentation`. This is necessary because loop body beats are stored inside `BeatKind::Loop { body: Vec<Beat> }` and are NOT in `timeline.beats`, so the existing `find_beat` call in `awaiting_cue_info()` would panic for loop body beats. `resume_cue()` clears both `awaiting_cue` and `awaiting_presentation`. Added `hop_index: Option<u32>` to `AwaitingCueInfo`; it is derived from `loop_stack.last().map(|f| f.hop_index)` — `Some(n)` inside a loop body, `None` for linear beats.

2. **cue_barrier.rs**: Added `hop_index: Option<u32>` to `CueBarrierStatus`. Updated `awaiting()` constructor to accept and store it. Updated `SuspendedTimeline::new()` to pass `awaiting.hop_index`. Updated the suspend log message to include `hop_index` for windowed telegraph chip diagnostics.

3. **assets/data/digimon/agumon/skills.ron**: Added `presentation: Some((cue_id: "agumon/baby_flame/bounce_hop", anim: Some("baby_flame_impact"), vfx: None, sfx: None))` to the `bounce_hop` loop body beat. This makes each bounce hop a presentation barrier in Windowed mode. HeadlessAuto is unaffected (ignores presentation).

4. **assets/digimon/agumon/anim_graph.ron**: Added self-transition `baby_flame_impact → baby_flame_impact` on `KernelCue` with `priority: Some((10))` so the impact animation loops on each hop cue, while the existing `TimeInNode → baby_flame_recover` fires when the loop ends naturally.

5. **tests/timeline/timeline_loop_hop_cue_parity.rs**: New test with 3 cases: (a) HeadlessAuto drains N=3 hops in one call; (b) Windowed suspends exactly 3 times, each `resume_cue()` advances one hop, `hop_index` in `AwaitingCueInfo` matches the current loop iteration, and the final intent stream matches HeadlessAuto; (c) extra `resume_cue()` after Done is a no-op.

6. **tests/timeline.rs**: Added `timeline_loop_hop_cue_parity` module per R003.

## Verification

Ran `cargo check` (pass), `cargo test --test timeline` (47/47 pass including 3 new tests), `cargo test` (all harnesses pass, zero failures), `cargo build --features windowed` (pass).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 17000ms |
| 2 | `cargo test --test timeline` | 0 | 47/47 pass — 3 new timeline_loop_hop_cue_parity tests green | 1000ms |
| 3 | `cargo test` | 0 | all harnesses pass, zero failures | 5000ms |
| 4 | `cargo build --features windowed` | 0 | pass | 4000ms |

## Deviations

none

## Known Issues

None.

## Files Created/Modified

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/cue_barrier.rs`
- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/timeline/timeline_loop_hop_cue_parity.rs`
- `tests/timeline.rs`
