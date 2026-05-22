---
id: T01
parent: S09
milestone: M002
key_files:
  - src/combat/observability/frame_time.rs
  - src/combat/observability/mod.rs
key_decisions:
  - Used f64::to_bits() for PartialEq on RegressionVerdict to avoid ordered_float dependency
  - 2ms absolute tolerance uses AND semantics with the 15% relative threshold — both must be exceeded to fail, protecting fast baselines
  - p95 uses ceil(0.95*N)-1 index on a sorted clone, consistent with common percentile conventions
  - FrameTimeStats stores f64 (up-cast from f32 samples) for precision in downstream formatting and comparison
duration: 
verification_result: passed
completed_at: 2026-05-22T08:10:14.642Z
blocker_discovered: false
---

# T01: Added pure FrameTimeAccumulator + RegressionVerdict in src/combat/observability/frame_time.rs with 9 green unit tests and re-exported symbols from the observability mod.

**Added pure FrameTimeAccumulator + RegressionVerdict in src/combat/observability/frame_time.rs with 9 green unit tests and re-exported symbols from the observability mod.**

## What Happened

Created src/combat/observability/frame_time.rs from scratch as a zero-dependency pure module (no Bevy, no wall-clock, no RNG). The module exposes: FrameTimeAccumulator (push f32 delta_secs, finalise() → FrameTimeStats), FrameTimeStats (count/mean_ms/p95_ms/max_ms/min_ms all in ms), format_frame_time_stats() emitting the parseable 'validation_frametime:' line, and frame_time_regression() implementing D027 thresholds (mean >15% AND >2ms absolute → fail; p95 >20% → fail). RegressionVerdict has four variants (Pass/MeanRegression/P95Regression/BothRegression) with PartialEq via f64::to_bits() to avoid pulling in ordered_float. The p95 implementation sorts a clone of the sample vec and indexes at ceil(0.95*N)-1. All symbols were declared as private submodule frame_time and re-exported from src/combat/observability/mod.rs. Nine inline #[cfg(test)] tests cover: empty accumulator (zeroed, no panic), single sample, p95 with an unsorted series, pass verdict, mean regression at/above the 2ms tolerance boundary, p95 regression, both-regression, and the absolute-tolerance protection on a fast baseline.

## Verification

cargo test --lib frame_time — 9 tests passed, 0 failed, exit code 0, duration 3.49 s compile + 0.00 s run.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib frame_time` | 0 | PASS — 9 tests: ok | 3540ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/observability/frame_time.rs`
- `src/combat/observability/mod.rs`
