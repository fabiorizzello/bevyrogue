---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: DrBag component + sum_dr helper + bootstrap insert already fully implemented in prior commit (2c09b85)

## Inputs

- None specified.

## Expected Output

- `src/combat/buffs.rs`
- `src/combat/mod.rs`
- `src/combat/bootstrap.rs`

## Verification

cargo check && cargo test --lib calculate_damage && cargo test bootstrap_spawn_composition
