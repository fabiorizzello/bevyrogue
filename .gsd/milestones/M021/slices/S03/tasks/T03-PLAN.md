---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Two-clock parity integration test: HeadlessAuto ≡ Windowed end-of-cast Intent stream

WHY: I3/D026 — both clocks must produce the same end-of-cast Intent stream; only timing differs. DO: create tests/timeline_two_clock_parity.rs reusing the branched-timeline pattern from T02 but add a `Presentation { cue_id, anim:None, vfx:None, sfx:None }` to at least one beat that fires intents (so the Windowed stall point exists). Run #1: a BeatRunner with default clock (HeadlessAuto) via run_to_completion over a fresh world → collect pending. Run #2: a BeatRunner built with `.with_clock(Clock::Windowed)` over an identically-spawned fresh world, driven by a MANUAL loop: call step in a loop; on StepOutcome::AwaitingCue assert it occurred at the presentation beat then call resume_cue() and continue; stop on Done/Halted (bounded max iterations to avoid hangs). Assert at least one AwaitingCue was observed (proves the stall is real, not bypassed) and normalize(headless_pending) == normalize(windowed_pending) using the same format!("{:?}") helper. Deterministic, no wall-clock. DONE-WHEN: cargo test --test timeline_two_clock_parity passes and asserts both the stall occurred and the streams are equal.

## Inputs

- `src/combat/api/runner.rs`
- `src/combat/api/clock.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/intent.rs`
- `tests/timeline_chain_bolt_port.rs`

## Expected Output

- `tests/timeline_two_clock_parity.rs`

## Verification

cargo test --test timeline_two_clock_parity 2>&1 | tail -5 (passes; asserts AwaitingCue observed and HeadlessAuto pending == Windowed pending)
