---
estimated_steps: 6
estimated_files: 4
skills_used: []
---

# T01: Create Agumon and Gabumon blueprints

Create the Agumon and Gabumon blueprint modules to process their respective custom signals. These blueprints will translate signals like `apply_heated` or `apply_deep_crack` into generic `twin_core_added_tag_transition` emissions.

Must-haves:
- `agumon::dispatch` handles `apply_heated`, `apply_meltdown_crack` and `apply_thermal_spark`.
- `gabumon::dispatch` handles `apply_chilled`, `apply_deep_crack` and `apply_thermal_spark`.
- Both blueprints registered in `src/combat/blueprints/mod.rs`.
- Ensure payload amounts map to `turns_left` where appropriate.

## Inputs

- `src/combat/blueprints/mod.rs`
- `src/combat/twin_core.rs`

## Expected Output

- `src/combat/blueprints/agumon.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/blueprints/mod.rs`

## Verification

cargo check
