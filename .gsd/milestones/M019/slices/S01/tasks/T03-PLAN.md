---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Wire DrBag through resolution.rs call sites + per-turn tick

## Inputs

- None specified.

## Expected Output

- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`

## Verification

cargo check && cargo test --test status_blessed_offensive && cargo test --test damage_breakdown_log
