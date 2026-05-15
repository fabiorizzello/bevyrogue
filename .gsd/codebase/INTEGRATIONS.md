# INTEGRATIONS

## Runtime integrations observed

## Third-party APIs and services

- **None observed in the game runtime.**
  - The Rust codebase does not show HTTP clients, SaaS SDKs, or remote API integrations in `Cargo.toml`.

## Databases

- **None observed.**
  - No database crates, migrations, or connection configuration were found.

## Authentication providers

- **None observed.**
  - No auth/OAuth/session provider integration is present.

## Infrastructure and deployment

- **Local filesystem assets**
  - Bevy loads gameplay data from `assets/data/*.ron`
  - Windowed and headless modes both rely on Bevy `AssetServer`
- **Local development toolchain**
  - Rust nightly + Cargo
  - Linux linker/toolchain tuning in `.cargo/config.toml`
- **No deployment platform configuration observed**
  - No Dockerfile, Vercel, Fly, Render, Netlify, Terraform, or Kubernetes manifests were visible in the scanned top-level tree.

## UI / platform integrations

- **Bevy windowed mode**
  - Optional desktop UI via `bevy_egui`
  - Enabled only through the `windowed` feature
- **Terminal CLI integration**
  - `src/bin/combat_cli.rs` uses `inquire` for interactive terminal menus

## Data and content integrations

- **RON asset integration**
  - `src/data/mod.rs` wires `bevy_common_assets::ron::RonAssetPlugin` for:
    - `UnitRoster`
    - `SkillBook`
    - `PartyConfig`
- **Sprite pipeline integration**
  - `tools/sprite_pipeline/` integrates local Python scripts, Blender assets, palettes, and model files to generate sprite atlases consumed from `assets/digimon/`

## Observability and file-based outputs

- **JSONL logging**
  - `src/combat/jsonl_logger.rs` is referenced by the headless runtime and CLI
  - Logging is env-gated via `BEVYROGUE_JSONL` per code comments and docs
- **Schedule graph dumping**
  - `examples/dump_schedule.rs` + `bevy_mod_debugdump` support schedule output into `.gsd/schedules/` per `Cargo.toml` comments

## Development-environment integrations

- **MCP / GSD workflow config**
  - `.mcp.json` defines a local `gsd-workflow` MCP server for project workflow tooling
  - This appears to be a development/agent integration, not an in-game runtime dependency

## External native/tool dependencies documented

From `docs/setup.md`, local builds may depend on:

- `mold`
- `pkg-config`
- `libasound2-dev`
- `libudev-dev`
- `libwayland-dev`
- `libxkbcommon-dev`
- **Blender 5.x** for sprite generation
- **Pillow** for Python image-processing scripts
- **ImageMagick** as an optional helper for the sprite pipeline
