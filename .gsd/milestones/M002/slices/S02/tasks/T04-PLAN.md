---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T04: Persist suspended timelines and resume only after cue release

Expected executor skills: bevy, rust-best-practices, rust-testing, tdd. Why: returning AwaitingCue from the runner is not enough; the turn pipeline must hold the runner and pending intents across frames so damage is not committed until the animation releases the barrier. Do: add a generic combat-runtime cue barrier resource/module (for example `TimelineClock`, `SuspendedTimelineState`, and a small release API) under `src/combat`, defaulting to `Clock::HeadlessAuto`. Refactor `run_timeline_backed_action` so it creates a Windowed runner when the resource requests `Clock::Windowed`, runs until Done/Halted/AwaitingCue, stores runner + pending intents + inflight metadata on AwaitingCue, leaves `CombatPhase::Resolving`, and applies the queued intents only after a release resumes the runner to Done. Factor the existing intent-application tail in `timeline_exec.rs` so both fresh and resumed executions use the same path. Add a continuation system/API that T05 can call from windowed animation; duplicate releases with no suspended timeline should be no-ops with a diagnostic. Add `tests/timeline_cue_barrier_pipeline.rs` to build a minimal Bevy app/world with a timeline-backed Basic action, set `TimelineClock(Clock::Windowed)`, emit the action, assert HP/log/events do not show damage while awaiting, release the cue, update again, and assert the same final damage/event stream as HeadlessAuto. Failure Modes (Q5): missing compiled timeline keeps existing OnActionFailed path; Halted runner emits failure and clears suspension; app shutdown/hot reload with a suspended timeline must not leave world-mutating partial state beyond buffered pending intents. Load Profile (Q6): one suspended timeline per action pipeline; 10x action spam should be rejected/ignored while CombatPhase is Resolving rather than accumulating unbounded runners. Negative Tests (Q7): duplicate release, release before any suspension, halted timeline, and no duplicate damage after resume. Done when the combat pipeline has a deterministic, inspectable two-clock barrier without any windowed feature dependency.

## Inputs

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/timeline.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/combat/turn_system/pipeline/mod.rs`
- `src/combat/turn_system/resolve.rs`
- `tests/timeline_two_clock_parity.rs`
- `tests/compiled_timeline_runtime_dispatch.rs`
- `assets/data/digimon/agumon/skills.ron`

## Expected Output

- `src/combat/runtime/cue_barrier.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/combat/turn_system/pipeline/mod.rs`
- `tests/timeline_cue_barrier_pipeline.rs`

## Verification

cargo test --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity --test compiled_timeline_runtime_dispatch

## Observability Impact

Adds an inspectable suspended-barrier state and diagnostics naming cast_id, skill_id, cue_id/beat_id, awaiting/released outcome, and duplicate/no-op release cases.
