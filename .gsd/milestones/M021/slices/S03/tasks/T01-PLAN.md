---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Clock-aware BeatRunner stepping: AwaitingCue + resume_cue + circuit-breaker log::warn!

WHY: I3/D026 needs HeadlessAuto vs Windowed to be provable, and F4 needs the circuit-breaker Halt to be observable. The runner currently has zero clock awareness and Halt returns silently. DO (per D005): (1) Add a `clock: Clock` field to `BeatRunner`, initialized to `Clock::HeadlessAuto` in `BeatRunner::new` (keep `new`'s signature unchanged so the 3 S02 integration tests and inline tests do not break); add `pub fn with_clock(mut self, clock: Clock) -> Self` builder. Import `crate::combat::api::clock::Clock`. (2) Add `StepOutcome::AwaitingCue` variant (StepOutcome already derives Debug/Clone/Copy/PartialEq/Eq — keep it). (3) Add internal latch state: `awaiting_cue: Option<BeatId>` (or equivalent) so a Presentation-bearing beat fires exactly once, then stalls. (4) In `step`, for both the linear path and the loop-body path: after `fire_beat` has run for a beat whose `presentation.is_some()`, if `self.clock == Clock::Windowed` and the beat is not yet resumed, return `StepOutcome::AwaitingCue` WITHOUT advancing `cursor`/`body_cursor` and WITHOUT re-evaluating edges; record the awaiting beat id. Do not re-fire that beat on the next `step`. (5) Add `pub fn resume_cue(&mut self)` clearing the latch so the subsequent `step` advances normally (computes next_beat / loop progression exactly as the non-presentation path would have). HeadlessAuto path must be byte-identical to today (never returns AwaitingCue). (6) In `run_to_completion`, handle the new `StepOutcome::AwaitingCue` arm by auto-calling `resume_cue()` and continuing the loop (so S02 drive-to-completion semantics are preserved even under Windowed). (7) Circuit breaker observability: at the `hop_index >= MAX_HOPS` branch (runner.rs:121-124), before returning `StepOutcome::Halted`, emit `bevy::log::warn!` including `self.cast_id`, `self.timeline.id`, and the hop count. Add `use bevy::log;` (applier.rs already uses this pattern). Do NOT emit an Intent (Intent::Reject deferred per D006 — keeps the stream clock/mode-independent). DONE-WHEN: inline unit tests in runner.rs cover: (a) HeadlessAuto regression — a Presentation-bearing timeline drives to Done with no AwaitingCue; (b) Windowed stalls — same timeline returns AwaitingCue exactly once at the presentation beat, fires its hook exactly once, and after resume_cue reaches Done; (c) HeadlessAuto pending == Windowed pending (normalized via format!("{:?}")) for that timeline; (d) existing 3 inline tests still pass. Full S02 integration suite and cargo check (both features) stay green.

## Inputs

- `src/combat/api/runner.rs`
- `src/combat/api/clock.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/mod.rs`

## Expected Output

- `src/combat/api/runner.rs`

## Verification

cargo test --lib combat::api::runner 2>&1 | tail -5 (inline runner tests incl. new headless/windowed/parity cases pass); cargo test --test timeline_onturnstart_kills --test timeline_chain_bolt_port --test timeline_validate_typo 2>&1 | tail -5 (S02 fixtures green, no regression); cargo check 2>&1 | tail -3 and cargo check --features windowed 2>&1 | tail -3 (exit 0, no new warnings)

## Observability Impact

Adds bevy::log::warn! on circuit-breaker Halt (cast_id + timeline id + hop count) — converts a silent return into a diagnosable signal. No Intent emitted, so stream parity invariants are unaffected.
