---
id: T02
parent: S03
milestone: M001
key_files:
  - tests/anim_validation.rs
key_decisions:
  - Kept the real-data adapter entirely in `tests/anim_validation.rs`, translating `aggregate_skill_book()` plus a project status vocabulary into generic `AnimationValidationCatalogs` so `src/animation` remains decoupled from `src/data` and Digimon-specific modules.
duration: 
verification_result: passed
completed_at: 2026-05-18T21:15:25.061Z
blocker_discovered: false
---

# T02: Added real Agumon animation validation tests that build generic catalogs from an external project-data adapter and fail loudly on missing catalog entries.

**Added real Agumon animation validation tests that build generic catalogs from an external project-data adapter and fail loudly on missing catalog entries.**

## What Happened

Extended `tests/anim_validation.rs` with a test-local adapter that consumes `bevyrogue::data::aggregate_skill_book()` and converts real project skill data into generic `AnimationValidationCatalogs`. The adapter fills particle and skill catalogs from real skill ids, merges status-bearing effects with a project status vocabulary so `Heated` is available for Agumon's graph, and keeps parameter catalogs generic/empty because the current Agumon graph uses only literal params. Added typed RON deserialization helpers for `assets/digimon/agumon/anim_graph.ron` and `assets/digimon/agumon/clip.ron`, a passing test proving the real Agumon graph validates through the external adapter seam, two negative tests that remove required adapter-provided particle/status entries and assert typed diagnostics with contextual node/field details, and a boundary guard test that rejects `crate::data`, `crate::combat`, or `digimon` coupling inside `src/animation/validation.rs`.

## Verification

Ran `cargo test --test anim_validation` through `gsd_exec`; all eight integration tests passed, including the new real-asset adapter validation, missing-particle/status negative cases, and the animation-boundary coupling guard.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cd /home/fabio/dev/bevyrogue && cargo test --test anim_validation` | 0 | ✅ pass | 685ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/anim_validation.rs`
