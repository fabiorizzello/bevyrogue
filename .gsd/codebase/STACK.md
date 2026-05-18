# Technology Stack

This document outlines the core technologies and libraries used in the `bevyrogue` project.

## Core Language & Runtime

- **Language**: [Rust](https://www.rust-lang.org/) (2024 Edition)
- **Engine**: [Bevy](https://bevyengine.org/) (v0.18.1)
  - Configured with a `headless-first` approach, decoupling core game logic from the rendering stack.
  - The `windowed` feature re-enables the rendering stack (winit, wgpu, etc.).

## Key Frameworks & Libraries

- **UI**: [egui](https://github.com/emilk/egui) via `bevy_egui` (v0.39.1)
- **Serialization**: 
  - [RON (Rusty Object Notation)](https://github.com/ron-rs/ron) (v0.8) for game data assets.
  - [Serde](https://serde.rs/) for mapping RON to Rust structures.
  - [bevy_common_assets](https://github.com/NiklasEi/bevy_common_assets) (v0.16) for custom RON asset loading.
- **RNG & Determinism**:
  - [bevy_rand](https://github.com/SvenSvenSven/bevy_rand) and `bevy_prng` (v0.14) using the `wyrand` algorithm.
  - Ensuring deterministic outcomes for combat simulations.
- **Error Handling**: [thiserror](https://github.com/dtolnay/thiserror) (v2)
- **Observability & Logging**:
  - [tracing](https://github.com/tokio-rs/tracing) and `tracing-subscriber` (v0.3) with JSON support.
- **CLI Utilities**: [inquire](https://github.com/mikaelmello/inquire) (v0.7) for interactive terminal prompts.
- **Type Safety**: `moonshine-kind` (v0.4.2) for kind-safe identifiers.

## Build & Development Tools

- **Build Tool**: `Cargo` (Rust's package manager and build system).
- **Profiles**:
  - `dev` profile uses the `cranelift` codegen backend for fast incremental builds on the main crate.
- **Testing**:
  - [insta](https://insta.rs/) for snapshot testing.
  - [bevy_mod_debugdump](https://github.com/jakobhellermann/bevy_mod_debugdump) (v0.15) for generating Bevy `Schedule` graph visualizations.
- **Test Runner**: `nextest` (configured in `.config/nextest.toml`).
