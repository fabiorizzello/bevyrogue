---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T01: Create degenerate stubs for non-Agumon digimon

Create degenerate anim_graph.ron and clip.ron for the other 5 Digimon (Gabumon, Dorumon, Renamon, Patamon, Tentomon) in their respective assets/digimon/<name>/ directories.

## Inputs

- `docs/future_design_draft/02-02b_animation_fsm.md`

## Expected Output

- `assets/digimon/gabumon/anim_graph.ron`
- `assets/digimon/dorumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `assets/digimon/patamon/anim_graph.ron`
- `assets/digimon/tentomon/anim_graph.ron`

## Verification

cargo test
