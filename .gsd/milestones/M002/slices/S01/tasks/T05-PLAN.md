---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T05: Runtime AnimGraph player + RenderPlugin/UiPlugin split (windowed Agumon idle)

## Inputs

- None specified.

## Expected Output

- `src/animation/player.rs`
- `src/animation/mod.rs`
- `src/animation/plugin.rs`
- `src/windowed.rs`
- `src/main.rs`
- `tests/anim_player_fsm.rs`

## Verification

cargo test --test anim_player_fsm
