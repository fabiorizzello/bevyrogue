---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Define the VfxAsset to enoki adapter

Implement an adapter that maps VfxAsset verbs/parameters (introspectable per D033) into the enoki effect representation the windowed renderer consumes. Cover the verbs Agumon/Renamon currently use. Warn-once on an unmapped verb.

## Inputs

- `src/animation/vfx_asset.rs`
- `src/animation/vfx.rs`
- `src/windowed/render.rs`

## Expected Output

- `An adapter turning VfxAsset into the enoki effect spec, warn-once on unmapped verbs`

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test (headless adapter test green)
