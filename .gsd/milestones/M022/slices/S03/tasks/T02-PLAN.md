---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Add contract tests with broken fixtures

Create tests/anim_fsm_validation.rs with test cases for each validator check. Use broken fixture strings to verify that errors are reported correctly.

## Inputs

- `src/combat/blueprints/anim_graph/validation.rs`

## Expected Output

- `tests/anim_fsm_validation.rs`

## Verification

cargo test --test anim_fsm_validation
