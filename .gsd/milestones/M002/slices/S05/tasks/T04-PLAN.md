---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T04: Per-hop kernel cue: visible loop iterations = kernel hop_index

## Inputs

- None specified.

## Expected Output

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/animation/player.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/timeline_loop_hop_cue_parity.rs`

## Verification

cargo test --test timeline_loop_hop_cue_parity --test timeline_two_clock_parity --test timeline_cue_barrier_pipeline --test anim_gameplay_command_forbidden --test anim_player_fsm --test bouncing_fire_off_baseline --test clip_atlas_parity
