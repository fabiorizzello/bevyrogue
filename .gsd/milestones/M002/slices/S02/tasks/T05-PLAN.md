---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T05: Wire windowed Sharp Claws playback and telegraph chip

## Inputs

- None specified.

## Expected Output

- `src/windowed.rs`
- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/widgets.rs`
- `src/ui/combat_panel/labels.rs`
- `tests/windowed_preview_cache.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache
