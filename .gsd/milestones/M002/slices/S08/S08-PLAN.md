# S08: Remediate graph purity and failure visibility

**Goal:** Remediate M002 validation gaps for graph purity and failure visibility: make the animation graph contract explicitly typed/input-driven and make timeline presentation failures observable, bounded, and non-corrupting.
**Demo:** After this: R009 has executable proof of typed pure graph input with no world globals or mutable graph context; R013 has structured failure visibility for cue timeout, missing skill-id, hot reload at next spawn, and dead target mid-loop.

## Must-Haves

- R009 has executable tests proving graph evaluation uses a closed typed input surface for caster/primary-target roles and has no mutable graph context or world-global read path.
- R013 has executable tests proving cue timeout forces a resume with structured diagnostic state, missing skill-id has a strict boot proof plus runtime fallback diagnostic, hot reload affects only the next spawned graph/player, and dead target mid-loop remains observable without branching presentation flow on liveness.
- Existing S01-S07 windowed/headless regression suites remain green.

## Proof Level

- This slice proves: contract + failure-path runtime proof

## Integration Closure

Consumes the S01-S07 animation graph, timeline runner, cue barrier, and windowed preview/registry seams. Provides auditable R009/R013 evidence for S09 milestone closeout; S09 still owns boundary-map and operational evidence packaging.

## Verification

- Adds or hardens durable diagnostic surfaces for presentation cue barriers and missing graph/skill fallback without logging secrets. Tests assert inspectable state/messages rather than relying on console-only output.

## Tasks

- [x] **T01: Add typed graph input purity contract** `est:1h`
  Introduce the smallest explicit graph-input/role seam needed for R009 and add animation-domain tests that prove roles are closed typed values rather than literals/world lookups. Keep AnimGraph data-only and deterministic: no mutable context object, no World parameter in graph/player evaluation, no ad-hoc Custom string escape hatch. Preserve current RON compatibility unless the tests prove a schema gap that must change.
  - Files: `src/animation/anim_graph.rs`, `src/animation/player.rs`, `tests/animation/anim_graph_input_purity.rs`, `tests/animation.rs`
  - Verify: cargo test --test animation anim_graph_input_purity

- [x] **T02: Make cue timeout force resume with structured state** `est:2h`
  Replace the current indefinite stuck-cue behavior with a bounded frame-budget timeout for windowed timeline barriers. The timeout must mark the barrier as timed out, persist enough structured context to inspect cast_id, skill_id, timeline, beat, cue, node/frame when known, request/resume the cue, and let combat continue without corrupting headless authority. Update stale R013 tests that currently document indefinite suspension.
  - Files: `src/combat/runtime/cue_barrier.rs`, `src/combat/turn_system/pipeline/timeline_exec.rs`, `src/combat/turn_system/mod.rs`, `tests/timeline/r013_failure_visibility.rs`, `tests/timeline/timeline_cue_barrier_pipeline.rs`
  - Verify: cargo test --test timeline r013_failure_visibility

- [x] **T03: Prove missing skill graph fallback and hot reload next spawn** `est:2h`
  Harden animation graph registry/player behavior so a missing skill graph is strict where M002 canonical assets are expected at boot, but runtime lookup failure degrades to a deterministic instant graph/player path with a structured diagnostic. Add a hot-reload test proving modified graph assets update registry state only for newly spawned players while an in-flight player keeps its current graph identity/state.
  - Files: `src/animation/registry.rs`, `src/animation/player.rs`, `src/animation/plugin.rs`, `tests/animation/anim_registry_failure_visibility.rs`, `tests/animation.rs`
  - Verify: cargo test --test animation anim_registry_failure_visibility

- [x] **T04: Close R013 dead-target mid-loop and regression sweep** `est:1h`
  Ensure the target-dead-mid-loop test asserts the presentation/runtime flow does not branch on liveness and that the event log makes the overshoot/death state inspectable. Run focused R009/R013 tests plus the previously affected windowed/headless suites to prove S08 did not regress S01-S07.
  - Files: `tests/timeline/r013_failure_visibility.rs`, `tests/timeline.rs`, `.gsd/REQUIREMENTS.md`
  - Verify: cargo test --features windowed --test animation --test timeline --test windowed_only

## Files Likely Touched

- src/animation/anim_graph.rs
- src/animation/player.rs
- tests/animation/anim_graph_input_purity.rs
- tests/animation.rs
- src/combat/runtime/cue_barrier.rs
- src/combat/turn_system/pipeline/timeline_exec.rs
- src/combat/turn_system/mod.rs
- tests/timeline/r013_failure_visibility.rs
- tests/timeline/timeline_cue_barrier_pipeline.rs
- src/animation/registry.rs
- src/animation/plugin.rs
- tests/animation/anim_registry_failure_visibility.rs
- tests/timeline.rs
- .gsd/REQUIREMENTS.md
