# Project

## What This Is

This project is a Rust Bevy combat prototype with typed RON data, headless-first tests, and optional `windowed` UI. The current planning focus is to turn the existing M022 asset-pipeline plan into M001: a cohesive animation asset and animation-FSM foundation built around `clip.ron`, `anim_graph.ron`, static validation, and hot reload.

## Core Value

The one thing that must work even if scope shrinks: animation assets must load and validate as typed, deterministic, headless-testable data before later visual runtime work depends on them.

## Project Shape

- **Complexity:** complex
- **Why:** The work crosses asset loading, validation, Bevy hot reload, future runtime boundaries, and an important architecture seam between generic animation engine code and Digimon-specific content.

## Current State

Existing combat and data systems already use typed RON assets and headless tests. `docs/M022/` contains a prior MILESTONE_PORTFOLIO-generated plan for the asset pipeline. This M001 uses that plan as the scope seed while adapting architecture to the current repo and the user-confirmed engine/content separation.

## Architecture / Key Patterns

- Rust + Bevy.
- Headless-first: `cargo check`, `cargo test`, and `cargo run` work without `windowed`; UI and watcher demos stay behind `--features windowed`.
- Existing data loading uses `RonAssetPlugin::<T>`, `AssetServer`, `LoadedWithDependencies`, and typed `DataError` surfaces.
- M001 introduces one cohesive animation module boundary for schema, loading, validation, orchestration, and later runtime/player behavior.
- Animation core must stay generic. Digimon-specific content belongs in assets, data, and adapter seams, not in core engine logic.
- Cross-asset validation should use explicit adapters into gameplay/data structures rather than hard direct coupling.

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [ ] M001: Animation asset pipeline foundation — Port and adapt M022 into a generic, roster-ready animation module with typed `clip.ron` and `anim_graph.ron`, validator §L, and real hot-reload proof.
