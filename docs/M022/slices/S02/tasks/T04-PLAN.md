---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Verify clip geometry parity

Add a contract test to verify that Agumon's clip.ron matches the geometry in agumon_atlas.json.

## Inputs

- `assets/digimon/agumon/clip.ron`
- `assets/digimon/agumon_atlas.json`

## Expected Output

- `tests/clip_geometry_parity.rs`

## Verification

cargo test --test clip_geometry_parity
