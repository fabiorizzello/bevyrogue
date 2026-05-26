---
estimated_steps: 3
estimated_files: 2
skills_used: []
---

# T01: Add bevy_enoki as a windowed-only dep and prove no headless leak with a dep-gating test

Why: The central S04 risk is dependency-graph leakage — bevy_enoki 0.6 hard-depends on the entire Bevy render stack (bevy_render, bevy_sprite_render, bevy_core_pipeline, bevy_camera, bevy_mesh, bevy_shader). If it is reachable from the default build, the headless dep-isolation requirements (R002/R005/R016) are violated and the headless build balloons. Retiring this risk FIRST, before any effect authoring, de-risks the whole spike.

Do: (1) In Cargo.toml [dependencies], add `bevy_enoki = { version = "0.6", optional = true }`. (2) Append `"dep:bevy_enoki"` to the existing `windowed` feature list (currently `windowed = ["dep:bevy_egui", "bevy/2d", "bevy/tonemapping_luts", "bevy/dynamic_linking"]`). Do NOT change the headless `bevy` feature set or the `dev`/`default` features. (3) Create a standalone headless test binary `tests/dependency_gating.rs` (its own single-binary domain per R003 — NOT under `tests/windowed_only/` and NOT `#![cfg(feature="windowed")]`, so it runs on the default `dev` headless build). The test shells out to `cargo tree` (via std::process::Command, with `--offline` to avoid network) twice: a headless invert query `cargo tree -e normal --no-default-features --features dev --invert bevy_enoki` MUST fail / match no package (proving absence), and a windowed invert query `cargo tree -e normal --no-default-features --features windowed --invert bevy_enoki` MUST succeed and contain `bevy_enoki` (proving presence only under windowed). Assert on exit status + stdout, log the captured output so a future agent can read the graph on failure. Do not assert exact phrasing of cargo's error. Skills: rust-development, cargo-nextest.

Done-when: headless `cargo build` does not compile bevy_enoki; `cargo test --test dependency_gating` passes (absence headless, presence windowed); `cargo build --features windowed` resolves bevy_enoki.

## Inputs

- `Cargo.toml`

## Expected Output

- `Cargo.toml`
- `tests/dependency_gating.rs`

## Verification

cargo test --test dependency_gating
