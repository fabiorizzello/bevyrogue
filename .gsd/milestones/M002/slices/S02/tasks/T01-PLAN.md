---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Expose deterministic cue-awaiting runner contract

Expected executor skills: rust-best-practices, rust-testing, tdd. Why: S02 depends on the kernel being able to stop at a presentation barrier instead of silently auto-resuming Windowed beats; current `BeatRunner::run_to_completion()` masks `StepOutcome::AwaitingCue`. Do: update `src/combat/runtime/runner.rs` so Windowed `run_to_completion()` returns `StepOutcome::AwaitingCue` when a presentation beat latches, while HeadlessAuto still drives to Done; keep `resume_cue()` as the only unlatch path and ensure the resumed beat hook is not re-fired. Update `tests/timeline_two_clock_parity.rs` from the current two-beat auto-resume fixture to an explicit cue-handshake fixture: HeadlessAuto completes; Windowed returns AwaitingCue at each presentation barrier; the test manually resumes; final `format!("{intent:?}")` streams are identical and terminate with Done. Failure Modes (Q5): if the runner exceeds max_steps, keep the existing panic safety net; if `resume_cue()` is called with no awaiting beat, it must be harmless or covered by existing semantics; malformed timelines remain covered by timeline validation rather than runner branching. Load Profile (Q6): per cast cost remains O(beats + loop hops); no shared mutable resources are introduced in this task. Negative Tests (Q7): assert no duplicated damage intent across AwaitingCue/resume and assert Windowed does not advance past the barrier without `resume_cue()`. Done when the runner contract is explicit, the parity test demonstrates both pause and identical final streams, and no windowed/wgpu code is needed.

## Inputs

- `src/combat/runtime/runner.rs`
- `tests/timeline_two_clock_parity.rs`
- `src/combat/runtime/timeline.rs`

## Expected Output

- `src/combat/runtime/runner.rs`
- `tests/timeline_two_clock_parity.rs`

## Verification

cargo test --test timeline_two_clock_parity

## Observability Impact

Clarifies StepOutcome docs and test failure messages around awaiting cue, resume, and duplicate intent prevention.
