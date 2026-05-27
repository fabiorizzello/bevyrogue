---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Drive Agumon and Renamon effects through VfxAsset

Repoint Agumon and Renamon windowed registration so their cast/impact effects are produced by the VfxAsset->enoki adapter instead of hand-registered enoki structs, making VfxAsset canonical (D052).

## Inputs

- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`
- `src/animation/vfx_asset.rs`

## Expected Output

- `Agumon and Renamon effects sourced from VfxAsset via the adapter`

## Verification

cargo test --features windowed --test windowed_only (green); manual cargo winx shows unchanged VFX
