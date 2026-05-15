---
id: T01
parent: S03
milestone: M021
key_files:
  - src/combat/api/runner.rs
key_decisions:
  - AwaitingCue latch uses two fields (awaiting_cue: Option<BeatId> + cue_just_resumed: bool) rather than precomputing next cursor, so the loop-body path (where next action is body_cursor advance, not a simple BeatId) is handled uniformly without duplication
  - Global stall gate placed before the loop_stack branch so a single check covers both linear and loop-body paths
  - run_to_completion auto-resumes AwaitingCue to preserve S02 batch-drive semantics unchanged
duration: 
verification_result: passed
completed_at: 2026-05-15T08:36:22.674Z
blocker_discovered: false
---

# T01: Added Clock-aware BeatRunner: AwaitingCue stall/resume_cue latch + circuit-breaker warn! on Halt

**Added Clock-aware BeatRunner: AwaitingCue stall/resume_cue latch + circuit-breaker warn! on Halt**

## What Happened

Implemented all 7 plan steps on `src/combat/api/runner.rs`:

1. **Clock field + with_clock builder**: Added `clock: Clock` (default `HeadlessAuto`) to `BeatRunner` and a `pub fn with_clock(mut self, clock: Clock) -> Self` builder. `new()` signature unchanged — S02 tests require no edits.

2. **StepOutcome::AwaitingCue**: Added the variant with a doc comment clarifying it is never returned by `run_to_completion` or HeadlessAuto.

3. **Latch state**: Added `awaiting_cue: Option<BeatId>` (set when stalling) and `cue_just_resumed: bool` (set by `resume_cue`, cleared at the next `step` entry to skip re-firing the already-fired beat).

4. **Stall logic — both paths**: At the top of `step`, a global stall gate (`if self.awaiting_cue.is_some() → return AwaitingCue`) covers both the loop-body and linear paths. After `fire_beat`, if `beat.presentation.is_some() && self.clock == Clock::Windowed`, we latch and return `AwaitingCue` without advancing `cursor`/`body_cursor`.

5. **resume_cue**: `pub fn resume_cue(&mut self)` clears `awaiting_cue` and sets `cue_just_resumed = true`. The subsequent `step` reads `cue_just_resumed`, skips `fire_beat`, and advances cursor/body_cursor normally.

6. **run_to_completion**: Added `StepOutcome::AwaitingCue => { self.resume_cue(); }` arm so batch-mode callers (S02 integration tests) transparently auto-resume.

7. **Circuit-breaker observability**: At `hop_index >= MAX_HOPS` in the loop-body path, added `bevy::log::warn!` carrying `cast_id`, `timeline.id`, and `hop_index` before returning `Halted`. Added `use bevy::log;` import.

Added 4 new inline tests (d–f): HeadlessAuto regression (no AwaitingCue despite Presentation), Windowed stall + exact-once hook fire + Done after resume, HeadlessAuto-vs-Windowed pending stream parity via `run_to_completion`.

## Verification

Ran `cargo test --lib combat::api::runner` → 6/6 pass (3 pre-existing + 3 new). Ran S02 integration fixtures: timeline_onturnstart_kills, timeline_chain_bolt_port, timeline_validate_typo → all green. `cargo check` and `cargo check --features windowed` both exit 0 with no new warnings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib combat::api::runner 2>&1 | tail -10` | 0 | 6 passed (headless_auto_presentation_reaches_done_no_awaiting_cue, windowed_stalls_on_presentation_hook_fires_once, headless_and_windowed_produce_identical_pending_stream + 3 pre-existing) | 1860ms |
| 2 | `cargo test --test timeline_onturnstart_kills --test timeline_chain_bolt_port --test timeline_validate_typo 2>&1 | tail -10` | 0 | 3 S02 fixtures pass, no regression | 2200ms |
| 3 | `cargo check 2>&1 | tail -3` | 0 | Finished dev profile, no new errors | 1630ms |
| 4 | `cargo check --features windowed 2>&1 | tail -3` | 0 | Finished dev profile, no new errors | 2460ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/api/runner.rs`
