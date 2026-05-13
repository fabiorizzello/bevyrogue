---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Migrate follow_up + in-tree tests to StatusBag

## Inputs

- None specified.

## Expected Output

- `src/combat/follow_up.rs`
- `src/combat/turn_system/tests.rs`

## Verification

`cargo check` clean across the whole tree. `cargo test --lib` green. Grep `rg 'StatusEffect\s*\{' src/` returns zero hits (all spawns go through `StatusBag::apply`).
