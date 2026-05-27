---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Define the VfxAsset to enoki adapter

Implement an adapter that maps VfxAsset verbs/parameters (introspectable per D033) into the enoki effect representation the windowed renderer consumes; the render-side consumption lives in render/spawn.rs after the S10 split. Cover the verbs Agumon/Renamon currently use. Warn-once on an unmapped verb using the shared warn-once util from S09.

## Inputs

- `src/animation/vfx_asset.rs`
- `src/animation/vfx.rs`
- `src/windowed/render/spawn.rs`

## Expected Output

- `An adapter turning VfxAsset into the enoki effect spec`
- `unmapped verbs warn once via the shared S09 util, naming the verb`

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test (headless adapter test green)
