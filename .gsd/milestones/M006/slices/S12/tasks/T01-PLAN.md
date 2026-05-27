---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Convert DetonateEffectRegistry to a keyed registry

Change DetonateEffectRegistry from a singleton slot to a keyed map (per species/effect id) in the registries module, updating Agumon's registration and the render consumer accordingly.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`

## Expected Output

- `DetonateEffectRegistry is keyed; Agumon registers under its key; consumer looks up by key`

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)
