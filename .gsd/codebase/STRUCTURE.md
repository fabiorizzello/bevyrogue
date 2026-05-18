# Directory Structure

This document outlines the organization of the `bevyrogue` codebase.

## Top-Level Layout

| Directory | Purpose |
|---|---|
| `assets/` | Game data (RON) and visual assets (atlases, sprites). |
| `docs/` | Project documentation and design drafts. |
| `examples/` | Standalone scripts (e.g., schedule graph generation). |
| `src/` | Primary source code. |
| `tests/` | Integration test suite. |
| `tools/` | Utility scripts and automation. |
| `.gsd/` | GSD metadata, decisions, and codebase scans. |

## Source Code Organization (`src/`)

- `main.rs`: Entry point for the game application.
- `lib.rs`: Library root, exposing public modules.
- `combat/`: The core gameplay logic.
  - `blueprints/`: Content-specific logic (per-Digimon/Enemy).
  - `kernel/`: Low-level combat primitives (Strain, Flow, Fatigue).
  - `runtime/`: Execution engine for skills and intents.
  - `turn_system/`: Turn order, AV gauge, and speed logic.
  - `mechanics/`: Specific combat rules (Damage, Status Effects, Toughness).
  - `observability/`: Logging, events, and validation snapshots.
- `data/`: Asset loading, validation, and data merging logic.
- `ui/`: UI components and rendering (gated behind the `windowed` feature).
- `bin/`: Additional executables.
  - `combat_cli.rs`: The CLI developer harness for combat simulation.

## Asset Organization (`assets/`)

- `assets/data/`:
  - `digimon/`: Skill and unit definitions for player-controlled units.
  - `enemies/`: Definitions for enemy units.
  - `party.ron`: Default starting party configuration.
- `assets/digimon/`: Sprite sheets and texture atlases.

## Test Organization (`tests/`)

- Functional integration tests are located in `tests/`.
- Naming convention: functional names (e.g., `twin_core_integration.rs`) rather than milestone-based IDs.
- Unit tests are typically inline within `src/` modules gated by `#[cfg(test)]`.
