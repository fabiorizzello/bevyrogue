---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Update renamon/kyubimon skills with precision signals.

Update Renamon and Kyubimon skill definitions in assets/data/skills.ron to use the new custom_signals instead of any remaining ad-hoc logic.

## Inputs

- `assets/data/skills.ron`

## Expected Output

- ``assets/data/skills.ron``

## Verification

cargo test --test digimon_signal_registry
