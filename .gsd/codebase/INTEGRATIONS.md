# External Integrations

This document summarizes the external systems and data sources the `bevyrogue` codebase interacts with.

## Data Assets

- **Local File System**: 
  - All game content (Digimon units, skills, party configurations) is stored as [RON](https://github.com/ron-rs/ron) files in the `assets/data/` directory.
  - Assets are loaded asynchronously using Bevy's `AssetServer`, with custom merging logic to aggregate partial definitions (e.g., merging unit definitions from multiple subdirectories).

## User Interfaces

- **CLI (Terminal)**: 
  - The `combat_cli` binary uses standard input/output for interaction.
  - Integrated with the `inquire` library for interactive party selection and encounter setup.
- **Windowed GUI**:
  - Optional integration with `winit` and `wgpu` via the `windowed` feature.
  - Uses `egui` for debugging and visual inspection overlays.

## Observability

- **JSONL Logging**:
  - Combat events can be captured and written as JSONL (JSON Lines) files for post-mortem analysis and validation.
- **Tracing**:
  - Integrated with the `tracing` ecosystem for structured logging to the terminal or diagnostic files.

## Infrastructure

- **Headless-First Design**:
  - The architecture is designed to run in CI/CD environments without a GPU or display, facilitating automated integration testing and simulation.
