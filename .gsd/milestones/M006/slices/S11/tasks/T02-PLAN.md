---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Cut over loaders to the catalog and remove the constants

Repoint asset loading to consume the catalog and delete DEFAULT_ANIM_GRAPH/CLIP/STANCE_PATHS. Ensure the existing roster loads identically. Warn-once on a folder missing a required asset.

## Inputs

- `src/animation/plugin.rs`

## Expected Output

- `DEFAULT_*_PATHS removed; loaders consume the catalog; warn-once on missing asset`

## Verification

cargo test (headless green); cargo test --features windowed --test windowed_only (green)
