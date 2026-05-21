---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T02: Sprite-anchored HP bar + damage-number HUD

## Inputs

- None specified.

## Expected Output

- `src/windowed/render.rs`
- `src/ui/combat_panel/display.rs`
- `src/ui/combat_panel/render.rs`
- `tests/windowed_preview_cache.rs`
- `tests/windowed_hud_hp_bar.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache --test windowed_hud_hp_bar
