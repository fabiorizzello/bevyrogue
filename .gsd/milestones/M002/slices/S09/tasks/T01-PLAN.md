---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Pure frame-time aggregator + threshold compare in lib observability

Why: the milestone's hard bar is 'no anim-graph-attributable frame-time regression vs a kernel-only baseline', but no frame-time measurement exists anywhere in the tree; the math must be deterministic and provable headlessly (R004) before it is wired into the display-dependent soak. Skills: tdd (write the unit tests first), design-an-interface (pure aggregator is the deep module; the windowed tick is a thin caller), observability. Do: add `src/combat/observability/frame_time.rs` exposing a `FrameTimeAccumulator` (push a per-frame delta in seconds) and a `FrameTimeStats { count, mean_ms, p95_ms, max_ms, min_ms }` finalizer, a `format_frame_time_stats(&FrameTimeStats, mode: &str) -> String` mirroring the `validation_snapshot:` prefix convention (emit `validation_frametime: ...`), and a `frame_time_regression(full: &FrameTimeStats, baseline: &FrameTimeStats) -> RegressionVerdict` implementing the D027 threshold (mean regression <=15%, p95 <=20%, 2ms absolute mean tolerance). Keep it pure — no Bevy types, no wall-clock reads, no RNG; the caller supplies deltas. Export the new symbols from `src/combat/observability/mod.rs`. Add inline `#[cfg(test)]` unit tests covering: empty accumulator (count=0, no panic / zeroed stats), single sample, p95 ordering with an unsorted series, and both pass and fail verdicts incl. the 2ms tolerance edge. Done-when: `cargo test --lib frame_time` is green and the symbols are re-exported from the observability mod.

## Inputs

- `src/combat/observability/mod.rs`
- `src/combat/observability/snapshot.rs`
- `src/combat/observability/format.rs`

## Expected Output

- `src/combat/observability/frame_time.rs`
- `src/combat/observability/mod.rs`

## Verification

cargo test --lib frame_time

## Observability Impact

Defines the parseable `validation_frametime:` line format and a structured RegressionVerdict used by the soak and the comparison artifact.
