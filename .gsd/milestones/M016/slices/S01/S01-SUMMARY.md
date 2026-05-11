---
id: S01
parent: M016
milestone: M016
provides:
  - A validated pattern for migrating other stateful mechanics (like Predator loop) to local blueprints.
requires:
  []
affects:
  []
key_files:
  - src/data/skills_ron.rs
  - src/combat/blueprints/tentomon.rs
  - tests/tentomon_blueprint.rs
key_decisions:
  - Added `TentomonCustomSignal` mapping directly to `BatteryLoopTransition` to decouple the logic.
  - Included `BatteryLoopSnapshot` in `ValidationSnapshot` to ensure the Battery Loop State is observable via CLI.
patterns_established:
  - Per-Digimon blueprint seam extracting `custom_signals` to Kernel Transitions without touching shared mechanics directly.
observability_surfaces:
  - ValidationSnapshot format string includes `battery_loop=...` output for CLI proofs.
drill_down_paths:
  - .gsd/milestones/M016/slices/S01/tasks/T01-SUMMARY.md
  - .gsd/milestones/M016/slices/S01/tasks/T02-SUMMARY.md
  - .gsd/milestones/M016/slices/S01/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-09T11:58:07.640Z
blocker_discovered: false
---

# S01: Tentomon/Kabuterimon Battery Loop Blueprint

**Migrated the Tentomon/Kabuterimon Battery Loop into a dedicated blueprint and decoupled its combat kernel resolution.**

## What Happened

Migrated the Battery Loop mechanics (static charge, circuit charge) from ad-hoc metadata into explicit declarative `custom_signals` in RON. Created `TentomonCustomSignal` mapped by the `tentomon.rs` blueprint to generic `BatteryLoopTransition`s in the combat kernel. Refactored the `ValidationSnapshot` to expose `BatteryLoopSnapshot` data, and verified the entire seam logic via an integration test.

## Verification

cargo check && cargo test --no-fail-fast and CLI output inspection.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None. Integrated custom signals directly from the skill definitions to trigger generic BatteryLoop transitions via a dedicated Tentomon blueprint.

## Known Limitations

None

## Follow-ups

None

## Files Created/Modified

- `src/data/skills_ron.rs` — Added `TentomonCustomSignal` enum and integrated it into `SkillCustomSignal`
- `assets/data/skills.ron` — Added `Tentomon` custom signals to `tentomon_basic`, `petit_thunder`, `mega_blaster`, etc.
- `src/combat/blueprints/mod.rs` — Added dispatch logic to handle `SkillCustomSignal::Tentomon`
- `src/combat/blueprints/tentomon.rs` — Implemented Tentomon blueprint transitions for the Battery Loop
- `src/combat/observability.rs` — Added Battery Loop observation fields to the Validation Snapshot
- `tests/tentomon_blueprint.rs` — Added test suite for the Tentomon blueprint seam
