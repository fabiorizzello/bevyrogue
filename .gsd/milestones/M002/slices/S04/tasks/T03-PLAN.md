---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T03: Project detonate transitions into a windowed flash indicator

## Inputs

- None specified.

## Expected Output

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/ui/combat_panel/render.rs`
- `tests/windowed_preview_cache.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache
