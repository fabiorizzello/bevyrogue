---
id: S09
parent: M002
milestone: M002
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - Placed BEVYROGUE_VALIDATION_BASELINE parser in lib crate (not binary) so integration tests can reach it (MEM030: tests/ link only against library crate)
  - Live windowed soak deferred per K001 (auto-mode must not launch windowed binary); D027 threshold math proven headlessly via T01/T02 unit tests
  - Baseline = skip entire RenderPlugin (camera + sprite spawn + anim-graph advance) for an apples-to-apples kernel-only comparison per D027
  - Boundary map rows cite actual on-disk test function names (extracted via grep) not just file paths, making the contract machine-checkable
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-22T08:32:24.808Z
blocker_discovered: false
---

# S09: Remediate validation evidence and operational closeout

**M002 closed out operationally: explicit producer→consumer boundary map (5 test-cited rows), VFX seam + skill-graph-extensibility integration proofs, pure frame-time aggregator with D027 threshold math + windowed-soak wiring, and a durable S09 closeout bundle referencing all S08 R009/R013 proofs.**

## What Happened

S09 closed out M002 operationally across five tasks.

**T01** added `src/combat/observability/frame_time.rs` — a pure `FrameTimeAccumulator`, `FrameTimeStats`, `format_frame_time_stats` (emits `validation_frametime: count=.. mean_ms=.. p95_ms=.. max_ms=.. min_ms=.. mode=full|baseline`), and `frame_time_regression` implementing the D027 threshold (mean ≤15%, p95 ≤20%, absolute mean ≤2ms). All 10 unit tests proven headlessly: empty accumulator, single sample, unsorted p95 ordering, pass/fail verdicts, 2ms tolerance edge case, and baseline-toggle string mapping. No Bevy types, no wall-clock reads, no RNG.

**T02** wired the accumulator into `WindowedValidationState` in `src/windowed/mod.rs`. `record_frame` pushes `Time::delta_secs()` each presentation frame after soak start; at the finish branch `format_frame_time_stats` is emitted before `AppExit::Success`. Added `BEVYROGUE_VALIDATION_BASELINE` env toggle: the parser (`parse_validation_baseline_toggle`) lives in lib crate (not binary) so integration tests can reach it, per MEM030. When set, `RenderPlugin` (camera + sprite spawn + `advance_agumon_presentation`) is skipped — an apples-to-apples kernel-only baseline per D027. Added `tests/windowed_only/frame_time_soak.rs` (2 tests green: baseline-toggle string mapping; known-delta-series stats + structured line). Artifacts `soak-console.log` and `frame-time-comparison.md` document the no-live-soak limitation and manual commands (K001: auto-mode must not launch windowed binary).

**T03** added `tests/animation/skill_graph_mapping_extensibility.rs` (3 tests green): multiple distinct non-default `AnimGraphId`s inserted into `SkillGraphRegistry` each resolve to their own graph with `source == Registry`; an unregistered id returns `source == InstantFallback` with a diagnostic; a stance-graph snapshot exposes a non-empty `graph().entry` for the `return_to_idle` boundary. Top-comment documents that `return_to_idle` lives in the windowed binary crate and is cited in the boundary map rather than callable from integration tests.

**T04** added `tests/animation/vfx_handle_seam.rs` (4 tests green): `SpawnParticle` round-trips losslessly through RON preserving opaque `ParticleId(String)` and closed `VfxLocus`/`VfxMotion` variants; unknown locus variant fails to deserialize (closed-enum guarantee); unknown motion variant fails to deserialize; serialized form contains no `Literal` and no ASCII digit when given a non-numeric `ParticleId` (no numeric gameplay payload leaks through the seam). Inline RON string literals for negative cases, matching `anim_gameplay_command_forbidden.rs` conventions.

**T05** assembled `M002-BOUNDARY-MAP.md` (5-row table: kernel skills.ron→timeline→anim-graph, anim-player cue→barrier→kernel resume, CombatEvent→§9 UI read-only, SkillGraphRegistry skill-id→windowed player + M003+ note, opaque ParticleId→windowed validate-only; each row cites actual test function names grepped from disk) and `S09-CLOSEOUT.md` (bundles boundary map link, S08 R009/R013 passing test names, soak artifacts, D027 verdict status, and regression-guard headless evidence). Boundary-map verification command confirmed BOUNDARY_MAP_AND_CLOSEOUT_OK with exit 0.

## Verification

Slice-level verification (all exit 0):

1. `cargo test --lib frame_time` → 10/10 passed (empty accumulator, single sample, p95 ordering, pass/fail verdicts, 2ms tolerance, baseline-toggle mapping, format prefix)
2. `cargo test --features windowed --test windowed_only frame_time` → 2/2 passed (baseline-toggle strings; known-delta-series stats + structured line)
3. `cargo test --test animation skill_graph_mapping_extensibility` → 3/3 passed (1:1 graph resolution, InstantFallback for unregistered, stance-entry non-empty)
4. `cargo test --test animation vfx_handle_seam` → 4/4 passed (RON round-trip, closed-enum rejection ×2, no numeric payload)
5. `cargo test --features windowed --test animation --test timeline --test windowed_only` → 25 windowed_only tests + animation + timeline, exit 0 (regression guard)
6. `cargo test --test animation clip_atlas_parity` → 2/2 passed (R003 geometry parity)
7. T05 boundary-map verification bash one-liner → BOUNDARY_MAP_AND_CLOSEOUT_OK (all 5 test paths resolve on disk, all 5 cited in boundary map, all 7 row keywords present)
8. All 4 artifacts verified on disk: soak-console.log, frame-time-comparison.md, M002-BOUNDARY-MAP.md, S09-CLOSEOUT.md

Known limitation: live windowed soak not run (K001); frame-time-comparison.md documents limitation and manual commands; D027 threshold math proven headlessly.

## Requirements Advanced

- R004 — Pure frame-time aggregator (T01) + soak wiring (T02) provide the measurement infrastructure for the D027 anim-graph frame-time regression bar; windowed build compiles with full soak path
- R005 — Skill-graph mapping extensibility test (T03) proves 1:1 AnimGraphId→AnimGraph resolution and the stance-return-to-idle entry boundary; VFX seam test (T04) proves the opaque ParticleId contract with no gameplay payload leak

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Live full-vs-baseline frame-time soak numbers are unmeasured (K001 / no display in auto-mode). frame-time-comparison.md has a pending results table to fill from a manual soak. SkillGraphRegistry still uses hardcoded skill-id wiring; data-driven registration is M003+ lift, boundary enforced.

## Follow-ups

None.

## Files Created/Modified

- `src/combat/observability/frame_time.rs` — Pure FrameTimeAccumulator, FrameTimeStats, format_frame_time_stats (validation_frametime: prefix), frame_time_regression (D027 thresholds), and parse_validation_baseline_toggle
- `src/combat/observability/mod.rs` — Re-exports FrameTimeAccumulator, FrameTimeStats, format_frame_time_stats, frame_time_regression, parse_validation_baseline_toggle
- `src/windowed/mod.rs` — WindowedValidationState gains FrameTimeAccumulator + record_frame; windowed_validation_tick accumulates per-frame deltas and emits validation_frametime: at finish; BEVYROGUE_VALIDATION_BASELINE toggle skips RenderPlugin for kernel-only baseline
- `tests/windowed_only/frame_time_soak.rs` — 2 windowed_only integration tests: baseline-toggle string mapping and known-delta-series stats + structured line
- `tests/windowed_only.rs` — Registered frame_time_soak via #[path]
- `tests/animation/skill_graph_mapping_extensibility.rs` — 3 tests: 1:1 multi-id registry resolution, InstantFallback for unregistered ids, non-empty stance-graph entry for return_to_idle boundary
- `tests/animation/vfx_handle_seam.rs` — 4 tests: SpawnParticle RON round-trip, closed VfxLocus/VfxMotion enum rejection, no numeric payload
- `tests/animation.rs` — Registered skill_graph_mapping_extensibility and vfx_handle_seam via #[path]
- `.gsd/milestones/M002/slices/S09/soak-console.log` — Documents no-live-soak limitation (K001) and manual soak commands
- `.gsd/milestones/M002/slices/S09/frame-time-comparison.md` — Frame-time comparison artifact: D027 threshold, pending results table, manual commands
- `.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md` — 5-row producer→consumer boundary map, each row citing an on-disk enforcing test function
- `.gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md` — Closeout bundle: S08 R009/R013 proofs, boundary map link, soak artifacts, D027 verdict, regression-guard results
