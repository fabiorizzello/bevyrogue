---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Convert DetonateEffectRegistry to a keyed registry

Change DetonateEffectRegistry from a singleton slot to a keyed map (per species/effect id) in render/registries.rs, updating Agumon's registration and the render/effects.rs consumer accordingly. On a keyed miss, warn once using the shared warn-once util from S09 (do not re-implement a dedup).

## Inputs

- `src/windowed/render/registries.rs`
- `src/windowed/render/effects.rs`
- `src/windowed/digimon/agumon/mod.rs`

## Expected Output

- `DetonateEffectRegistry is keyed; Agumon registers under its key; consumer looks up by key`
- `a keyed miss warns once via the shared S09 util`

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)
