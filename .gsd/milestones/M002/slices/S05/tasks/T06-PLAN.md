---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T06: Twin Core synergy badge + slice verification matrix

## Inputs

- None specified.

## Expected Output

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/labels.rs`
- `src/windowed/mod.rs`
- `tests/windowed_twin_core_badge.rs`

## Verification

cargo test --features windowed --test windowed_twin_core_badge --test windowed_preview_cache --test windowed_hud_hp_bar --test windowed_target_hurt
