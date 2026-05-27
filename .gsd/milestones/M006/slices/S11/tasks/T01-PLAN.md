---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Define the catalog discovery source of truth

Introduce a data-driven catalog that enumerates Digimon presentation assets (anim_graph, clip, stance) from the on-disk/asset layout instead of the DEFAULT_*_PATHS constants. Deterministic ordering, no wall-clock, no unseeded RNG (R004).

## Inputs

- `src/animation/plugin.rs`
- `src/animation/registry.rs`
- `assets/data/party.ron`

## Expected Output

- `A deterministic catalog that discovers per-Digimon presentation asset paths from data`

## Verification

cargo test (headless green); cargo check
