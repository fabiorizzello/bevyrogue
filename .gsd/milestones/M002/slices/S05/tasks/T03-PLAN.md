---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T03: OnHitTaken → frame-counted target blink/hurt projection

## Inputs

- None specified.

## Expected Output

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/windowed/mod.rs`
- `src/windowed/render.rs`
- `tests/windowed_preview_cache.rs`
- `tests/windowed_target_hurt.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache --test windowed_target_hurt
