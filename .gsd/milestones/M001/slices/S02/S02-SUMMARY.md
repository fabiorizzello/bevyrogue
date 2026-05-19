---
id: S02
parent: M001
milestone: M001
provides:
  - Typed `Clip` schema and loader registration under the generic animation module.
  - Agumon `clip.ron` authored asset with exact geometry parity proof against the source atlas.
  - Clip readiness semantics and inspection surfaces that S03 validators can consume.
requires:
  []
affects:
  - S03
  - S04
key_files:
  - src/animation/clip.rs
  - src/animation/mod.rs
  - tests/clip_parse.rs
  - assets/digimon/agumon/clip.ron
  - tests/clip_geometry_parity.rs
  - src/animation/plugin.rs
  - tests/clip_asset.rs
key_decisions:
  - Use a generic `Clip` schema inside the shared `src/animation` module rather than Digimon-specific code.
  - Treat atlas range counts as inclusive when proving parity with source JSON.
  - Only mark clip readiness true after the typed `Clip` is readable from `Assets<Clip>`, mirroring graph readiness semantics.
patterns_established:
  - Typed animation RON assets should use strict schema parsing with loud failures for unknown fields.
  - Asset readiness for typed animation resources should be gated on successful `Assets<T>` reads, not just load events.
  - Authoritative source-data parity tests should assert exact geometry plus inclusive range-count semantics to catch drift.
observability_surfaces:
  - `AnimationClipLoadState` readiness resource for typed clip assets.
  - `AnimationClipHandles` inspection surface for headless tests.
  - Asset load/modify event handling in `AnimationAssetPlugin` that exposes premature-readiness failures through tests.
drill_down_paths:
  - .gsd/milestones/M001/slices/S02/tasks/T01-SUMMARY.md
  - .gsd/milestones/M001/slices/S02/tasks/T02-SUMMARY.md
  - .gsd/milestones/M001/slices/S02/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-18T21:01:07.068Z
blocker_discovered: false
---

# S02: S02

**Shipped a generic typed `Clip` asset path, authored Agumon `clip.ron`, and proved lossless geometry parity plus Bevy asset-readiness behavior in headless tests.**

## What Happened

S02 extended the generic `src/animation` boundary with a strict typed `Clip` schema and exported it for downstream validator work. The slice then authored `assets/digimon/agumon/clip.ron` from the existing atlas source and added a drift-detection parity test that checks exact geometry, animation ranges, and inclusive count semantics against `assets/digimon/agumon_atlas.json`. Finally, the animation asset plugin was wired to register typed clip RON assets, maintain clip handles/load state, and gate readiness on successful `Assets<Clip>` reads so clip loading mirrors the graph-loader contract established in S01.

## Verification

Fresh closeout verification passed for every planned check: `cargo test --test clip_parse` (valid parse plus malformed/unknown-field rejection), `cargo test --test clip_geometry_parity` (exact Agumon geometry/range parity and inclusive count semantics), `cargo test --test clip_asset` (typed Bevy asset loading and readiness not flipping early), and `cargo test` (full regression suite).

## Requirements Advanced

- R003 — Added the typed `Clip` schema, authored Agumon `clip.ron`, and wired typed Bevy asset loading plus readiness checks.
- R008 — Kept clip parsing, parity verification, and asset-load verification fully headless via deterministic tests.

## Requirements Validated

- R003 — `cargo test --test clip_parse`, `cargo test --test clip_geometry_parity`, `cargo test --test clip_asset`, and `cargo test` all passed, proving typed `clip.ron` loading and exact Agumon geometry parity against `assets/digimon/agumon_atlas.json`.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

- none — None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

This slice does not yet validate graph-to-clip references or non-Agumon roster assets, and it does not prove live windowed hot reload or runtime playback behavior.

## Follow-ups

S03 should consume the exported `Clip` types and clip readiness semantics to implement adapter-based graph/clip validation with typed diagnostics. S04 should extend authored `clip.ron` coverage beyond Agumon and perform the manual `windowed` hot-reload proof.

## Files Created/Modified

- `src/animation/clip.rs` — Defined the strict typed `Clip` schema and supporting geometry/range types.
- `src/animation/mod.rs` — Exported clip types from the generic animation module.
- `tests/clip_parse.rs` — Added direct schema parse tests for valid input, unknown-field rejection, and malformed range rejection.
- `assets/digimon/agumon/clip.ron` — Authored Agumon clip geometry and ranges from the existing atlas source data.
- `tests/clip_geometry_parity.rs` — Added exact parity and inclusive count checks against `assets/digimon/agumon_atlas.json`.
- `src/animation/plugin.rs` — Registered typed clip assets and clip readiness tracking in `AnimationAssetPlugin`.
- `tests/clip_asset.rs` — Added typed Bevy asset-load and readiness-gating smoke coverage for Agumon clip assets.
