---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Teach AnimGraphPlayer KernelCue and author Sharp Claws cue graph

## Inputs

- None specified.

## Expected Output

- `src/animation/player.rs`
- `tests/anim_player_fsm.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_graph_asset.rs`

## Verification

cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity
