---
id: S01
parent: M001
milestone: M001
provides:
  - Generic animation module seam and typed `AnimGraph` schema.
  - Closed graph vocabulary for nodes, transitions, predicates, commands, parameter references, and target shapes.
  - Loader registration and readiness lifecycle for `anim_graph.ron` assets.
  - Real Agumon `anim_graph.ron` coverage through the typed loading path.
requires:
  []
affects:
  - S03
  - S04
key_files:
  - src/lib.rs
  - src/animation/mod.rs
  - src/animation/anim_graph.rs
  - src/animation/plugin.rs
  - assets/digimon/agumon/anim_graph.ron
  - tests/anim_graph_parse.rs
  - tests/anim_graph_asset.rs
key_decisions:
  - Use closed serde enums for commands, predicates, playback/target vocabulary so unknown schema values fail at parse time.
  - Model transition destinations with an explicit `TransitionTarget::Node(...) | Exit` enum to avoid ambiguous untagged RON decoding.
  - Only mark animation graph readiness after both a qualifying asset event and successful typed asset lookup from `Assets<AnimGraph>`.
patterns_established:
  - Generic typed asset loaders in Bevy should gate readiness on both asset events and successful `Assets<T>` readability.
  - Keep animation schema/loading under a cohesive `src/animation` seam instead of scattering asset types through data or Digimon-specific modules.
observability_surfaces:
  - `AnimationGraphLoadState.ready` as the boot-time health signal for graph availability.
  - Deterministic typed parse failures and failing asset tests as the primary failure signal in the headless verification lane.
drill_down_paths:
  - .gsd/milestones/M001/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M001/slices/S01/tasks/T02-SUMMARY.md
  - .gsd/milestones/M001/slices/S01/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-18T20:51:37.747Z
blocker_discovered: false
---

# S01: S01

**Verified the new generic animation module, closed typed AnimGraph schema, and real Agumon anim_graph.ron loader path with passing headless parse, asset, and full regression tests.**

## What Happened

S01 established the first generic animation-module seam under src/animation and exposed it from src/lib.rs, then proved the first real asset path end to end with a closed typed AnimGraph contract. The slice introduced serde-backed enums for graph vocabulary so unknown node-adjacent shapes fail during parsing instead of leaking stringly-typed values deeper into runtime code. It also added a real assets/digimon/agumon/anim_graph.ron fixture for baby_flame using generic graph data only, plus an AnimationAssetPlugin that registers typed RON loading and tracks configured graph handles through a Bevy-style load-state resource. Slice closeout re-ran all planned headless verification and confirmed the loader only reports ready once the typed asset is both event-observed and readable from Assets<AnimGraph>, preserving deterministic failure behavior for malformed or out-of-vocabulary content.

Gates to close: Q3 Threat Surface — limited to local RON asset input; malformed files can deny boot/test success but do not introduce external IO or secret exposure in this slice. Q4 Requirement Impact — advances R001 and validates the typed-loading contract in R002 while preserving the headless-first constraint behind R008. Q5 Failure Modes — unknown command, predicate, and target-shape variants fail deterministically in parse tests, and ready state does not silently flip before typed asset readability. Q6 Load Profile — graph assets are boot-time parsed in small counts, so current cost is one typed RON parse per graph with scale risk primarily in asset count and log volume rather than runtime CPU. Q7 Negative Tests — dedicated parse tests cover out-of-vocabulary schema values, and the asset test proves the real Agumon graph becomes a readable AnimGraph asset. Q8 Operational Readiness — current health signal is AnimationGraphLoadState.ready plus Bevy asset events; failure signal is deterministic cargo-test/asset-load failure; recovery is to fix the bad graph RON and rerun headless tests; monitoring remains lightweight until later validation/runtime slices add richer diagnostics.

## Verification

Fresh closeout verification passed on the required slice checks: `cargo test --test anim_graph_parse` passed with 5 tests covering valid typed parsing and rejection of unknown command, predicate, and target-shape variants; `cargo test --test anim_graph_asset` passed, proving `assets/digimon/agumon/anim_graph.ron` loads through `AnimationAssetPlugin` and only flips readiness after the typed asset is readable; `cargo test` passed for the full headless repository regression, confirming the new animation module integrates cleanly without breaking existing coverage.

## Requirements Advanced

- R001 — Established the public `src/animation` module seam and kept AnimGraph schema/loading generic rather than Digimon-specific.
- R008 — Kept the slice fully headless and verified schema plus asset loading exclusively through cargo test flows.

## Requirements Validated

- R002 — Fresh closeout runs of `cargo test --test anim_graph_parse`, `cargo test --test anim_graph_asset`, and `cargo test` proved typed `anim_graph.ron` loading through the animation module with closed schema rejection of out-of-vocabulary values.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

This slice proves typed graph schema and headless loading only; clip loading, validator semantics, non-Agumon roster coverage, and windowed hot reload remain for later slices.

## Follow-ups

S02 should add typed clip loading and geometry parity proof. S03 should layer validator diagnostics and adapter-based cross-asset checks onto the typed graph and clip contracts. S04 should extend the same path to non-Agumon assets and complete the manual windowed hot-reload demonstration.

## Files Created/Modified

- `src/lib.rs` — Exported the new public animation module seam.
- `src/animation/mod.rs` — Declared animation module structure and surfaced shared schema/plugin types.
- `src/animation/anim_graph.rs` — Defined the closed typed AnimGraph schema and parse contract types.
- `src/animation/plugin.rs` — Registered typed RON loading and animation graph readiness tracking.
- `assets/digimon/agumon/anim_graph.ron` — Added the real Agumon baby_flame animation graph fixture using generic data only.
- `tests/anim_graph_parse.rs` — Added parse-contract tests for valid loading and unknown-variant rejection.
- `tests/anim_graph_asset.rs` — Added headless asset-loading verification for the real Agumon graph.
