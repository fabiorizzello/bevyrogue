# S09 frame-time comparison — full vs kernel-only baseline

Milestone M002 / Slice S09 / Task T02.

## Purpose

Measure the anim-graph/render-attributable frame-time cost of the windowed app by
comparing two windowed soak runs that differ **only** in the anim-graph/render
path, and check the result against the D027 regression bar.

## Method (D027)

- A pure, headless aggregator (`FrameTimeAccumulator` → `FrameTimeStats`,
  `bevyrogue::combat::observability`) collects `Time::delta_secs()` from the
  windowed presentation tick (`windowed_validation_tick`), starting one frame
  after soak start.
- **Full run:** default windowed app (anim-graph/render path active).
- **Baseline run:** `BEVYROGUE_VALIDATION_BASELINE=1` — `register()` skips
  `render::RenderPlugin` (camera + sprite spawn + `advance_agumon_presentation`),
  the only registered difference. Everything else (DefaultPlugins, soak tick,
  exit path) is identical, giving an apples-to-apples comparison.
- At soak finish each run emits a parseable line mirroring the
  `validation_snapshot:` convention:
  `validation_frametime: mode=full|baseline count=.. mean_ms=.. p95_ms=.. max_ms=.. min_ms=..`

## D027 pass bar (anim-graph-attributable regression, full vs baseline)

- mean regression ≤ 15 %  **OR** mean absolute delta ≤ 2 ms (low-sample grace floor)
- p95 regression ≤ 20 %

Both the mean relative threshold AND the 2 ms absolute floor must be exceeded
for a mean failure (`frame_time_regression` / `RegressionVerdict`).

## Measurement status: NO LIVE SOAK CAPTURED IN AUTO-MODE

KNOWLEDGE rule **K001** forbids auto-mode from launching the windowed binary, and
no display is available to the auto-mode executor. The live full-vs-baseline soak
is therefore a **manual** step (see `soak-console.log` for the exact commands).

What IS proven now, headlessly and deterministically (R004-compatible):

| Proof | Command | Result |
|-------|---------|--------|
| Aggregator math (count/mean/p95/max/min, empty/edge cases) | `cargo test --lib frame_time` | 10 passed |
| Regression verdict vs D027 thresholds | `cargo test --lib frame_time` | covered (pass/mean/p95/both, 2 ms floor) |
| Baseline toggle string mapping + soak accumulator wiring + structured line | `cargo test --features windowed --test windowed_only frame_time` | 2 passed |
| Windowed build incl. baseline register seam | `cargo build --features windowed` | exit 0 |
| Baseline toggle threaded into `WindowedValidationConfig` | `cargo test --features windowed --bin bevyrogue windowed_validation` | 4 passed |

The aggregator that produces the live numbers is the same code unit-proven above;
the only unexercised-in-CI portion is the live winit frame source, which is
inherently non-deterministic and display-dependent and so is excluded from the
headless suite by design.

## Results table (fill in after the manual soak)

| Metric  | Full (mode=full) | Baseline (mode=baseline) | Δ abs | Δ % | Within D027? |
|---------|------------------|--------------------------|-------|-----|--------------|
| count   | _pending_        | _pending_                | —     | —   | —            |
| mean_ms | _pending_        | _pending_                | _pending_ | _pending_ | _pending_ (≤15% OR ≤2 ms) |
| p95_ms  | _pending_        | _pending_                | _pending_ | _pending_ | _pending_ (≤20%) |
| max_ms  | _pending_        | _pending_                | _pending_ | —   | —            |

**Verdict: PENDING manual soak** — paste the two `validation_frametime:` lines
from `soak-console.log`, compute the deltas, and record Pass / MeanRegression /
P95Regression / BothRegression per `frame_time_regression`.
