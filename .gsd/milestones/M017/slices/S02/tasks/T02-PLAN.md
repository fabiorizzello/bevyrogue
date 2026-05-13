---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Migrate apply pipeline to StatusBag

## Inputs

- None specified.

## Expected Output

- `src/combat/turn_system/pipeline.rs`
- `src/combat/bootstrap.rs`

## Verification

`cargo check` compiles cleanly for the apply path. Manual read: `OnStatusApplied` still fires on refresh; `OnStatusResisted` still gated by `roll_pct`.
