# STRUCTURE

## Top-level layout

```text
.
├── src/                  Rust application/library code
├── tests/                Rust integration tests
├── assets/               Runtime data assets and sprite atlases
├── docs/                 Project documentation
├── examples/             Example/dev utilities
├── tools/                Ancillary tooling (notably sprite pipeline)
├── .cargo/               Cargo config and aliases
├── .gsd/                 GSD workflow/project metadata
├── Cargo.toml            Rust package manifest
├── Cargo.lock            Cargo lockfile
├── rust-toolchain.toml   Pinned Rust toolchain
├── CLAUDE.md             Repo-specific agent onboarding
└── .mcp.json             Local MCP server config
```

## Source code organization

## `src/`

- `lib.rs` — public module exports
- `main.rs` — primary app entrypoint
- `headless.rs` — default headless app wiring
- `windowed.rs` — optional egui/windowed wiring
- `party_validation.rs` — party config validation logic
- `bin/combat_cli.rs` — separate CLI harness binary

## `src/combat/`

Primary gameplay package. Key subareas:

- `api/` — framework/runtime extension points, registries, intent/signal/timeline infrastructure
- `blueprints/` — per-identity combat behavior modules
- `turn_system/` — turn-pipeline implementation split across files
- flat modules for mechanics and state such as:
  - `state.rs`
  - `turn_order.rs`
  - `resolution.rs`
  - `damage.rs`
  - `toughness.rs`
  - `status_effect.rs`
  - `events.rs`
  - `observability.rs`
  - `plugin.rs`

## `src/data/`

Typed asset schemas and load-time compilation:

- `mod.rs` — data plugin / asset loading orchestration
- `party_ron.rs` — party config schema
- `skills_ron.rs` — skill schema and validation
- `skill_timeline.rs` — timeline compilation logic
- `units_ron.rs` — roster/unit schema

## `src/ui/`

- `mod.rs`
- `combat_panel.rs`

This module is only relevant in `windowed` builds.

## Test organization

## `tests/`

- Large integration-test suite with one file per behavior/capability
- Naming is functional rather than milestone-based, per `CLAUDE.md` and `tests/README.md`
- `tests/common/mod.rs` provides shared helpers/fixtures

Examples of test groupings present:

- roster/bootstrap validation
- action/turn pipeline behavior
- status effects and follow-ups
- blueprint runtime proofs
- scenario balance/TTK checks
- boundary and observability contracts

## Asset organization

## `assets/data/`

Canonical gameplay content:

- `units.ron`
- `skills.ron`
- `party.ron`

## `assets/digimon/`

Generated/consumed sprite atlases:

- `*_atlas.png`
- `*_atlas.json`

## Tooling organization

## `tools/sprite_pipeline/`

Contains a separate local asset-production toolchain:

- `scripts/` — Python and shell scripts
- `configs/` — per-character pipeline configs
- `palettes/` — palette files
- `plugins/` — Blender plugin assets
- `raw_models/` — source 3D models
- `references/` — visual references
- `standards/` — per-character standards/scoring docs
- `output/` — generated pipeline output

## Documentation locations

- `CLAUDE.md` — working conventions for coding agents
- `docs/combat_current.md` — current combat architecture entrypoint
- `docs/setup.md` — environment and tooling setup
- `docs/research/`, `docs/future_design_draft/` — longer-form supporting docs
- `doc/` and `target/doc/` — generated Rust documentation artifacts

## Configuration file locations

- `Cargo.toml` — package/dependency/features/profiles
- `Cargo.lock` — locked dependency versions
- `rust-toolchain.toml` — pinned nightly toolchain + component
- `.cargo/config.toml` — env, linker, rustflags, unstable flags, Cargo aliases
- `.mcp.json` — local MCP server wiring
- `.claude/` — agent-related local configuration
- `.gsd/` — project workflow state and generated planning artifacts

## Library vs binary structure

This repository is organized as:

- **one primary library crate surface** (`src/lib.rs`)
- **one main binary** (`src/main.rs`)
- **one additional binary** (`src/bin/combat_cli.rs`)
- **one example** (`examples/dump_schedule.rs`)
