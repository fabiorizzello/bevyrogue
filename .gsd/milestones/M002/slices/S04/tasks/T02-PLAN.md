---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Register Agumon Baby Burner detonate with headless tests

## Inputs

- None specified.

## Expected Output

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/agumon/baby_burner.rs`
- `tests/agumon_baby_burner_reactive.rs`
- `tests/common/app.rs`

## Verification

cargo test --test agumon_baby_burner_reactive --test unit_died_payload
