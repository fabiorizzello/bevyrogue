---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T02: Wire frame-time aggregation + baseline toggle into the windowed soak and capture artifacts

Why: with the pure aggregator proven, the soak must actually accumulate per-frame deltas and support a kernel-only baseline run so the milestone FPS bar can be measured (research deliverable #5, highest risk). Skills: observability, verify-before-complete. Do: in `src/windowed/mod.rs` give `WindowedValidationState` a `FrameTimeAccumulator`; in `windowed_validation_tick` push `Time::delta_secs()` each frame after start and, at the existing finish branch, emit `format_frame_time_stats(...)` (mode `full` or `baseline`) right before `AppExit::Success`. Add a `BEVYROGUE_VALIDATION_BASELINE` env toggle parsed like `parse_windowed_validation_toggle`; when set, skip registering the anim-graph/render-driving systems (kernel-only baseline) while leaving the soak/exit path intact — the only difference between the two runs is the anim-graph/render path. Add `tests/windowed_only/frame_time_soak.rs` (register in `tests/windowed_only.rs` with a `#[path]` line) that does NOT read any .gsd artifact: assert the baseline-toggle parser maps the documented truthy/falsey strings, and that feeding a known delta series through the accumulator wired into `WindowedValidationState` yields the expected stats (use a small public helper to record into the state's accumulator). Then build `--features windowed`, run the soak in both modes if a display is available, capturing stdout to `.gsd/milestones/M002/slices/S09/soak-console.log`, and write `.gsd/milestones/M002/slices/S09/frame-time-comparison.md` recording full vs baseline mean/p95/max and the explicit pass/fail against the D027 threshold; if no display is available, record that limitation explicitly in the comparison artifact and rely on the T01 unit proof. Done-when: windowed build compiles, the windowed_only frame_time test is green, and both .gsd artifacts exist with the structured line (or documented no-display note).

## Inputs

- `src/windowed/mod.rs`
- `src/combat/observability/frame_time.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/phase_strip_readonly.rs`

## Expected Output

- `src/windowed/mod.rs`
- `tests/windowed_only/frame_time_soak.rs`
- `tests/windowed_only.rs`
- `.gsd/milestones/M002/slices/S09/soak-console.log`
- `.gsd/milestones/M002/slices/S09/frame-time-comparison.md`

## Verification

cargo build --features windowed && cargo test --features windowed --test windowed_only frame_time

## Observability Impact

Emits `validation_frametime: count=.. mean_ms=.. p95_ms=.. max_ms=.. min_ms=.. mode=full|baseline` at soak finish and persists the captured console + comparison as durable evidence.
