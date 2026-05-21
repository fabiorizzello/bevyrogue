---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T04: Agumon Stance FSM asset + whole-sheet clip range + stance load path

## Inputs

- None specified.

## Expected Output

- `assets/digimon/agumon/clip.ron`
- `assets/digimon/agumon/stance.ron`
- `src/animation/plugin.rs`
- `tests/anim_stance_asset.rs`

## Verification

cargo test --test anim_stance_asset --test clip_geometry_parity
