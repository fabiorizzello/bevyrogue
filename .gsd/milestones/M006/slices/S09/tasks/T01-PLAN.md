---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Carve registries and types into render/registries.rs

Move the engine-generic registry structs/resources and shared presentation types out of render.rs into a new src/windowed/render/registries.rs, re-exporting as needed so external call sites keep compiling. No logic edits.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`

## Expected Output

- `src/windowed/render/registries.rs holds the moved registries/types; render.rs imports them`

## Verification

cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)
