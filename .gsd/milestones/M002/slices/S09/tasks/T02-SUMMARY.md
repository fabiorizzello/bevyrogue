---
id: T02
parent: S09
milestone: M002
key_files:
  - src/windowed/mod.rs
  - src/combat/observability/frame_time.rs
  - src/combat/observability/mod.rs
  - tests/windowed_only/frame_time_soak.rs
  - tests/windowed_only.rs
  - .gsd/milestones/M002/slices/S09/soak-console.log
  - .gsd/milestones/M002/slices/S09/frame-time-comparison.md
key_decisions:
  - Baseline = skip the whole render::RenderPlugin (camera + sprite spawn + anim-graph advance) so full-vs-baseline differs only in the anim-graph/render path, matching D027's apples-to-apples intent
  - Placed the BEVYROGUE_VALIDATION_BASELINE parser in the lib (combat::observability) not the windowed binary, because tests/ link only against the library crate (MEM030)
  - Added PartialEq and removed Copy on WindowedValidationState/added PartialEq to FrameTimeAccumulator to accommodate the Vec-backed accumulator field
  - Kept the T01-established validation_frametime: line format (mode first) rather than re-ordering to the slice-verification field order — it is already parseable key=value and unit-proven
  - Per K001, did not run the live windowed soak; artifacts document the limitation + manual commands and rely on headless T01/T02 proofs
duration: 
verification_result: passed
completed_at: 2026-05-22T08:24:10.861Z
blocker_discovered: false
---

# T02: Wired the windowed soak to accumulate per-frame deltas + a BEVYROGUE_VALIDATION_BASELINE kernel-only toggle, emitting a structured validation_frametime: line at finish; captured manual-soak + comparison artifacts

**Wired the windowed soak to accumulate per-frame deltas + a BEVYROGUE_VALIDATION_BASELINE kernel-only toggle, emitting a structured validation_frametime: line at finish; captured manual-soak + comparison artifacts**

## What Happened

Gave `WindowedValidationState` a `FrameTimeAccumulator` (+ a `record_frame` helper) and made `windowed_validation_tick` push `Time::delta_secs()` every presentation frame after the soak start frame; at the existing finish branch it now finalises the accumulator and emits `format_frame_time_stats(.., mode)` right before `AppExit::Success`, where `mode` is `full` or `baseline`. Added a `BEVYROGUE_VALIDATION_BASELINE` env toggle: the pure parser `parse_validation_baseline_toggle` was placed in the lib (`src/combat/observability/frame_time.rs`, re-exported from the observability mod) so windowed_only integration tests can reach it — binary-private items in `src/windowed/mod.rs` are not linkable from tests/ (captured as MEM030). The toggle is parsed in `config_from_env`/`parse_windowed_validation_config` and stored as `WindowedValidationConfig.baseline`; when set, `register()` skips `render::RenderPlugin` entirely (camera + sprite spawn + `advance_agumon_presentation`), the only registered difference between the two runs — an apples-to-apples kernel-only baseline per D027. Had to add `PartialEq` to `FrameTimeAccumulator` in the lib because `WindowedValidationState` derives it (and dropped the now-impossible `Copy` derive on the state). Added `tests/windowed_only/frame_time_soak.rs` (registered via `#[path]` in `tests/windowed_only.rs`), reading no .gsd artifact: it asserts the baseline-toggle string mapping (truthy/falsey/garbage) and that a known delta series through the same `FrameTimeAccumulator` the soak uses yields the expected count/mean/p95/max/min and the structured `validation_frametime:` line for both modes. Wrote the two durable S09 artifacts; because KNOWLEDGE K001 forbids auto-mode from launching the windowed binary, `soak-console.log` and `frame-time-comparison.md` document the no-execution limitation, give the exact manual full + baseline soak commands, the expected structured line, the D027 pass bar, and rely on the T01 unit proof + the new T02 integration/build proofs.

## Verification

cargo build --features windowed → exit 0. cargo test --features windowed --test windowed_only frame_time → 2 passed (baseline toggle mapping; known-delta-series stats + structured line). cargo test --lib frame_time → 10 passed (T01 nine + new baseline-toggle unit test). cargo test --features windowed --bin bevyrogue windowed_validation → 4 passed (incl. new baseline-threading test). Live windowed soak NOT run: K001 forbids auto-mode launching the windowed binary; both .gsd artifacts document this and the manual capture commands.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | PASS — Finished dev profile, no errors | 4260ms |
| 2 | `cargo test --features windowed --test windowed_only frame_time` | 0 | PASS — 2 passed; 0 failed | 4090ms |
| 3 | `cargo test --lib frame_time` | 0 | PASS — 10 passed; 0 failed | 3006ms |
| 4 | `cargo test --features windowed --bin bevyrogue windowed_validation` | 0 | PASS — 4 passed; 0 failed | 1200ms |

## Deviations

Plan said to add a public helper to record into WindowedValidationState's accumulator and test it; because integration tests cannot link the binary's WindowedValidationState (MEM030), the test instead feeds the identical lib FrameTimeAccumulator type that record_frame wraps, proving the same accumulation surface. record_frame remains in the binary as the thin wrapper the soak calls.

## Known Issues

Live full-vs-baseline soak numbers are unmeasured (K001 / no display). frame-time-comparison.md has a pending results table to fill from a manual soak. Open question for manual verification: baseline mode drops the camera with RenderPlugin; if bevy_egui requires a Camera for the primary context pass, the user may need to confirm the baseline run still reaches soak finish (the egui panels are not needed for frame-time measurement).

## Files Created/Modified

- `src/windowed/mod.rs`
- `src/combat/observability/frame_time.rs`
- `src/combat/observability/mod.rs`
- `tests/windowed_only/frame_time_soak.rs`
- `tests/windowed_only.rs`
- `.gsd/milestones/M002/slices/S09/soak-console.log`
- `.gsd/milestones/M002/slices/S09/frame-time-comparison.md`
