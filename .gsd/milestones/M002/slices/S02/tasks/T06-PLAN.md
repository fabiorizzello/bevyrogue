---
estimated_steps: 1
estimated_files: 18
skills_used: []
---

# T06: Run full S02 verification and close integration regressions

## Inputs

- None specified.

## Expected Output

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/animation/player.rs`
- `src/windowed.rs`
- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/widgets.rs`
- `src/ui/combat_panel/labels.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/data/digimon/agumon/skills.ron`
- `assets/data/digimon/agumon/unit.ron`
- `tests/timeline_two_clock_parity.rs`
- `tests/anim_player_fsm.rs`
- `tests/anim_graph_asset.rs`
- `tests/agumon_sharp_claws_asset.rs`
- `tests/timeline_cue_barrier_pipeline.rs`
- `tests/windowed_preview_cache.rs`

## Verification

cargo build --features windowed
