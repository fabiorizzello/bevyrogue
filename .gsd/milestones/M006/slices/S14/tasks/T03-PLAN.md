---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Adapter round-trip and coverage test

Add a headless test that parses a VfxAsset and asserts the adapter produces the expected enoki spec for the covered verbs, and that an unmapped verb triggers the warn path. Locks D052's source-of-truth contract.

## Inputs

- `src/animation/vfx_asset.rs`

## Expected Output

- `Headless test proving VfxAsset compiles to the expected enoki spec and warns on unmapped verbs`

## Verification

cargo test --features windowed --test windowed_only (adapter test green)
