---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T01: Split render into per-concern submodules

Carve render.rs into src/windowed/render/{playback,spawn,effects,feedback,clock}.rs, each owning one concern, with render/mod.rs as the orchestrator. Move systems verbatim where possible; no behavior change to clock catch-up or barrier release.

## Inputs

- `src/windowed/render.rs`

## Expected Output

- `render decomposed into playback/spawn/effects/feedback/clock submodules behind render/mod.rs`

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)
