---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Cross-species no-collision test

Add a test where two species register distinct detonate/effect entries and assert each resolves to its own effect (no overwrite), proving the keyed registry fixed the singleton collision.

## Inputs

- `src/windowed/render.rs`

## Expected Output

- `Test proving two species keep independent effect entries`

## Verification

cargo test --features windowed --test windowed_only (no-collision case green)
