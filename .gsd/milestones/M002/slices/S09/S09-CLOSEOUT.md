# S09 Closeout — M002 operational closeout evidence bundle

Milestone M002 / Slice S09 / Task T05.

This bundle ties together the milestone-level closeout evidence required by
M002's hard acceptance: the R009/R013 proofs from S08, the producer→consumer
boundary map, the captured soak console output, the frame-time baseline
comparison, and the regression-guard command result. It **references** existing
proofs — it does not re-validate them.

## 1. R009 / R013 validation (from S08 — referenced, not re-run)

S08 closed the two M002 remediation gaps and marked R009 and R013 validated with
executable proof. See `.gsd/milestones/M002/slices/S08/S08-SUMMARY.md` (and the
S08 UAT) for full detail. Passing tests cited there:

- **R009 — typed pure graph input.** `cargo test --test animation
  anim_graph_input_purity` (closed `AnimGraphRole`/`AnimGraphInput` seam,
  read-only input, no world-global or mutable-context escape hatch), reinforced
  by the windowed regression sweep.
- **R013 — timeout / fallback / hot-reload / dead-target failure visibility.**
  `cargo test --test timeline r013_failure_visibility` (cue timeout
  force-resume + dead-target post-KO observability), `cargo test --test
  animation anim_registry_failure_visibility` (missing-graph instant fallback +
  boot diagnostics + hot-reload-next-spawn), and `cargo test --features
  windowed --test animation --test timeline --test windowed_only`.

These are cited by name only; this slice does not re-execute them.

## 2. Boundary map

See `.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md` for the explicit
producer→consumer boundary map. It documents all five required seams
(kernel→timeline→anim-graph; anim player→cue barrier→kernel resume;
`CombatEvent`→§9 UI/HUD read-only; `SkillGraphRegistry` skill-id→windowed
player, incl. the hardcoded-constant constraint to lift for M003+; opaque
`ParticleId` VFX seam→windowed validate-only consumer), each citing an enforcing
test that exists on disk.

## 3. Captured soak console output

See `.gsd/milestones/M002/slices/S09/soak-console.log`.

**Status: NOT captured in auto-mode by design.** KNOWLEDGE rule **K001** forbids
auto-mode from launching the windowed binary, and no display is available to the
auto-mode executor. The log file documents the exact manual commands (full run +
`BEVYROGUE_VALIDATION_BASELINE=1` baseline run) and the expected structured
`validation_frametime:` line at soak finish. The frame-time aggregation and
baseline-toggle code paths that produce those lines **are** proven headlessly
(see §5).

## 4. Frame-time comparison verdict

See `.gsd/milestones/M002/slices/S09/frame-time-comparison.md`.

- **Method (D027):** a pure headless `FrameTimeAccumulator` →
  `FrameTimeStats` aggregator collects per-frame `Time::delta_secs()` from the
  windowed presentation tick; the full run vs the `BEVYROGUE_VALIDATION_BASELINE=1`
  baseline run differ **only** in the anim-graph/render path
  (`render::RenderPlugin`), giving an apples-to-apples regression measurement.
- **Pass bar:** mean regression ≤ 15 % **OR** mean absolute Δ ≤ 2 ms (low-sample
  grace floor); p95 regression ≤ 20 %. Both the relative threshold and the 2 ms
  floor must be exceeded for a mean failure.
- **Live verdict: PENDING manual soak.** The live winit frame source is
  non-deterministic and display-dependent, so it is excluded from the headless
  suite by design and must be captured manually (K001). The aggregator/verdict
  math itself is fully proven headlessly (§5).

## 5. Regression-guard command result (headless, deterministic)

These are the headless proofs of the frame-time aggregation, regression verdict,
and baseline-toggle wiring — the same code unit that produces the live soak
numbers. Results as recorded in `frame-time-comparison.md` (T01/T02):

| Proof | Command | Result |
|-------|---------|--------|
| Aggregator math (count/mean/p95/max/min, empty/edge cases) | `cargo test --lib frame_time` | 10 passed |
| Regression verdict vs D027 thresholds (pass / mean / p95 / both, 2 ms floor) | `cargo test --lib frame_time` | covered |
| Baseline toggle + soak accumulator wiring + structured line | `cargo test --features windowed --test windowed_only frame_time` | 2 passed |
| Windowed build incl. baseline register seam | `cargo build --features windowed` | exit 0 |
| Baseline toggle threaded into `WindowedValidationConfig` | `cargo test --features windowed --bin bevyrogue windowed_validation` | 4 passed |

## Closeout status

- R009 / R013: **validated** in S08 (referenced above).
- Boundary map: **present**, five required rows, every cited test on disk.
- Soak console + live frame-time verdict: **manual step** deferred per K001;
  generating code is headlessly proven.
- M002 operational closeout evidence is assembled in this slice directory.
