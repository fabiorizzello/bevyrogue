---
id: S08
parent: M002
milestone: M002
provides:
  - Executable validation evidence for R009 typed pure graph input.
  - Executable validation evidence for R013 timeout/fallback/hot-reload/dead-target failure visibility.
  - Downstream-ready summary for S09 milestone closeout packaging.
requires:
  - slice: S01
    provides: Animation graph/player/registry foundations and typed schema surfaces.
  - slice: S02
    provides: Two-clock cue barrier and timeline execution seams that S08 hardened for timeout/failure visibility.
  - slice: S06
    provides: Hot-reload and windowed preview seams that S08 converted into executable observability proof.
  - slice: S07
    provides: Recently stabilized animation/timeline/windowed baselines that S08 regressed against.
affects:
  - S09
key_files:
  - src/animation/anim_graph.rs
  - src/animation/player.rs
  - src/animation/registry.rs
  - src/animation/plugin.rs
  - src/combat/runtime/cue_barrier.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - tests/animation/anim_graph_input_purity.rs
  - tests/animation/anim_registry_failure_visibility.rs
  - tests/timeline/r013_failure_visibility.rs
  - tests/timeline/timeline_cue_barrier_pipeline.rs
key_decisions:
  - Use a closed read-only AnimGraphRole/AnimGraphInput seam as the R009 contract, while keeping legacy player entrypoints as default-input wrappers.
  - Cue timeout recovery must resume through the same released-runner path as normal cue release and preserve structured post-timeout diagnostic state.
  - Animation players bind cloned resolved-graph snapshots so hot reload updates registry state only for future spawns/resolutions, not in-flight players.
patterns_established:
  - Closed typed input lenses are the preferred seam for animation graph evaluation; no world-global or mutable graph-context escape hatch is allowed.
  - Failure-visibility paths should leave inspectable structured state after recovery rather than only transient console output.
  - Hot reload for presentation assets should be next-spawn by snapshot binding when mid-flight mutation would risk state corruption.
observability_surfaces:
  - CueBarrierStatus timeout fields and last_message/last_status diagnostic retention.
  - AnimationGraphLookupDiagnostics runtime fallback reporting.
  - AnimationGraphLoadState boot failure visibility.
  - ActionLog plus CombatEvent assertions for post-KO overshoot observability.
drill_down_paths:
  - .gsd/milestones/M002/slices/S08/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002/slices/S08/tasks/T02-SUMMARY.md
  - .gsd/milestones/M002/slices/S08/tasks/T03-SUMMARY.md
  - .gsd/milestones/M002/slices/S08/tasks/T04-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-21T22:15:22.785Z
blocker_discovered: false
---

# S08: Remediate graph purity and failure visibility

**Validated the typed pure AnimGraph input seam and hardened failure-visibility behavior for cue timeout, missing graph fallback, hot reload, and dead-target mid-loop observability without regressing prior M002 work.**

## What Happened

S08 closed the two explicit remediation gaps called out for M002. On the graph-contract side, the animation layer now exposes a closed typed input seam through AnimGraphRole and a read-only AnimGraphInput set, and the animation test harness proves graph evaluation stays data-only and deterministic: typed roles deserialize, stringly or unknown roles are rejected, explicit input is read without mutation, and the legacy player entrypoints remain thin default-input wrappers rather than opening a world-read or mutable-context escape hatch. On the failure-visibility side, the cue barrier now has a bounded 180-frame timeout with inspectable structured status that preserves cast, skill, timeline, beat, cue, hop, and animation context, then force-resumes through the same release path used by normal cue completion so headless authority is not corrupted. The animation registry/runtime path proves a missing skill graph is strict and inspectable at boot for canonical assets while runtime lookup degrades deterministically to an instant fallback with structured diagnostics. Hot reload behavior is locked to next spawn by binding players to cloned resolved-graph snapshots, so registry updates affect only newly resolved players while in-flight players keep their current graph identity/state. The dead-target-mid-loop regression was strengthened to prove presentation flow does not branch on liveness: the same cast continues emitting observable post-KO overshoot in both CombatEvent output and ActionLog. Slice closeout also advanced requirement bookkeeping by marking R009 and R013 validated with executable proof. I attempted a fresh-context reviewer subagent for closeout, but the reviewer agent was unavailable because the environment had hit its usage limit; slice closure therefore relies on direct verification evidence and task summaries.

## Verification

Fresh slice verification passed via gsd_exec run 96671a46-5c9d-451d-b0b3-b193c11a2c90, which executed all slice-plan checks: cargo test --test animation anim_graph_input_purity; cargo test --test timeline r013_failure_visibility; cargo test --test animation anim_registry_failure_visibility; and cargo test --features windowed --test animation --test timeline --test windowed_only. Additional task-level evidence from prior focused runs remained green: T01 purity harness (53ff245c-6c54-4856-8a87-e04307696626 / f6c71a7d-85f4-4aaa-a7c6-151fc5db5804), T02 failure-visibility harness (584d4abe-164b-4337-af04-701cc5d17ade) plus cue-barrier pipeline regression recorded in the task summary, T03 registry failure-visibility harness (0844e945-3c39-4571-8f94-3047b42bc60a), and T04 full regression sweep (3e34c568-f148-4106-9af1-773d8701d635). Outcomes proved: closed typed graph input with no world-global path, structured cue timeout force-resume, deterministic missing-graph fallback plus boot diagnostics, hot reload only at next spawn, dead-target overshoot observability, and no regressions across the affected animation/timeline/windowed suites.

## Requirements Advanced

- R009 — Closed the validation gap by supplying executable typed-input purity proof and recording the seam as a data-only contract.
- R013 — Closed the remediation gap by hardening structured failure visibility for cue timeout, missing graph fallback, hot reload next spawn, and dead-target mid-loop observability.

## Requirements Validated

- R009 — cargo test --test animation anim_graph_input_purity plus the windowed regression sweep prove closed typed roles, read-only input handling, and no world-global or mutable graph-context path.
- R013 — cargo test --test timeline r013_failure_visibility, cargo test --test animation anim_registry_failure_visibility, and cargo test --features windowed --test animation --test timeline --test windowed_only prove timeout force-resume, structured fallback diagnostics, next-spawn hot reload, and dead-target observability.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None in the implementation contract. Closeout attempted a reviewer subagent pass, but the agent was unavailable due environment usage limits; direct verification evidence was used instead.

## Known Limitations

This slice proves the contract and failure paths primarily through automated tests rather than a new manual windowed demo. Reviewer subagent feedback could not be obtained in this session because the delegated agent was unavailable.

## Follow-ups

S09 should package these validated R009/R013 proofs into milestone-level closeout evidence alongside the boundary map, console output capture, and frame-time baseline evidence.

## Files Created/Modified

- `src/animation/anim_graph.rs` — Defines the closed typed AnimGraphRole and AnimGraphInput seam used for graph purity proof.
- `src/animation/player.rs` — Threads read-only typed input through player advancement while preserving legacy default-input wrappers.
- `src/animation/registry.rs` — Provides structured graph lookup diagnostics, deterministic instant fallback, and resolved-graph snapshot behavior.
- `src/animation/plugin.rs` — Exposes inspectable boot-time animation graph load-state failures.
- `src/combat/runtime/cue_barrier.rs` — Adds bounded timeout accounting, structured timeout status, and force-resume recovery state.
- `src/combat/turn_system/pipeline/timeline_exec.rs` — Ticks cue-barrier timeouts and resumes through the normal released-runner path.
- `tests/animation/anim_graph_input_purity.rs` — Proves typed input purity, role closure, and legacy/default-input parity.
- `tests/animation/anim_registry_failure_visibility.rs` — Proves missing-skill fallback diagnostics, boot failure visibility, and hot-reload-next-spawn behavior.
- `tests/timeline/r013_failure_visibility.rs` — Proves cue timeout force-resume and dead-target post-KO observability without liveness branching.
- `tests/timeline/timeline_cue_barrier_pipeline.rs` — Exercises the integrated cue-timeout recovery path in the timeline pipeline.
