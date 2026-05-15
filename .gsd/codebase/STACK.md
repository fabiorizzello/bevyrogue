# STACK

## Languages and runtimes

- **Rust** — primary application language (`edition = "2024"` in `Cargo.toml`)
- **Rust nightly** — pinned in `rust-toolchain.toml`
- **Python 3** — used by `tools/sprite_pipeline/scripts/*.py`
- **Shell** — helper scripts in `tools/sprite_pipeline/scripts/*.sh`

## Core runtime stack

- **Bevy `0.18.1`** — game engine / ECS runtime
  - Built **headless by default** with `default-features = false`
  - Enabled features in default build: `std`, `async_executor`, `multi_threaded`, `bevy_asset`, `bevy_log`, `bevy_state`, `file_watcher`
- **bevy_egui `0.39.1`** — optional windowed UI, behind feature `windowed`
- **bevy_common_assets `0.16`** with `ron` — loads RON assets into typed Bevy assets
- **serde `1`** + `derive` — serialization/deserialization
- **serde_json `1`** — JSON/JSONL output surfaces
- **ron `0.8`** — canonical game-data format under `assets/data/*.ron`
- **rand `0.8`** — randomness support
- **inquire `0.7`** — terminal prompts for `src/bin/combat_cli.rs`
- **moonshine-kind `0.4.2`** — entity instance helpers used in gameplay code

## Build and dev tooling

- **Cargo** — package manager and build tool
- **rustup** — implied toolchain manager for the pinned nightly setup (`docs/setup.md`)
- **Cranelift codegen backend** — enabled for the workspace dev profile via `rustc-codegen-cranelift-preview`
- **LLVM fallback for dependencies** — configured in `[profile.dev.package."*"]`
- **mold** — configured as the Linux dev linker in `.cargo/config.toml`
- **`-Zshare-generics=y`** — nightly rustflag in `.cargo/config.toml`
- **Cargo aliases** in `.cargo/config.toml`
  - `cargo dev`
  - `cargo build-dev`
  - `cargo check-dev`
  - `cargo test-dev`
  - `cargo winx`

## Features and build modes

- **Default build:** headless runtime for agents / CI
- **`windowed` feature:** pulls UI/render stack (`bevy/2d` + `bevy_egui`)
- **`dev` feature:** enables `bevy/dynamic_linking`

## Test and debugging stack

- **Rust integration tests** under `tests/`
- **bevy_mod_debugdump `0.15`** — dev dependency for schedule graph dumping (`examples/dump_schedule.rs`)
- **Rustdoc output** is present in `doc/` and `target/doc/`

## Data and asset formats

- **RON** — unit roster, skill book, and party config (`assets/data/units.ron`, `skills.ron`, `party.ron`)
- **PNG + JSON atlases** — Digimon sprite assets in `assets/digimon/`

## Ancillary pipeline tooling

The repository also contains a sprite-generation toolchain under `tools/sprite_pipeline/`:

- Python scripts for render / atlas assembly
- Blender `.blend` plugin assets
- Raw `.fbx` / `.glb` source models
- Optional external tools documented in `docs/setup.md` and `tools/sprite_pipeline/GETTING_STARTED.md`
