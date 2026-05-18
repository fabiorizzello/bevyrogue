---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Wire validator to boot-time finish()

Update CombatPlugin or DataPlugin to run the validator during finish() or after load, emitting DataError if validation fails.

## Inputs

- `src/data/mod.rs`

## Expected Output

- `src/combat/plugin.rs`
- `src/data/mod.rs`

## Verification

cargo test --test anim_fsm_validation
