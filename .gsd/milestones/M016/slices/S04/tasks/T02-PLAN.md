---
estimated_steps: 5
estimated_files: 2
skills_used: []
---

# T02: Migrate Twin Core skills to custom signals and verify

Update `skills.ron` to attach Twin Core custom signals to Agumon and Gabumon's skills, and assert in tests that action resolution correctly generates Twin Core tags.

Must-haves:
- Add `custom_signals` to `pepper_breath`, `agumon_follow_up`, `agumon_ult`.
- Add `custom_signals` to `bubble_blast`, `gabumon_follow_up`, `gabumon_ult`.
- Ensure tests verify that resolving these skills emits the correct `CombatKernelTransition::Tag` addition.

## Inputs

- `assets/data/skills.ron`
- `tests/twin_core_integration.rs`

## Expected Output

- `assets/data/skills.ron`
- `tests/twin_core_integration.rs`

## Verification

cargo test --test twin_core_integration
