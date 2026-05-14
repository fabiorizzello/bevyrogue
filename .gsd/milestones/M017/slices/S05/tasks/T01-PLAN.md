---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: Add cleanse-immune regression test for Blessed

Lock in the §H.1 cleanse-immune line for Blessed as a slice-level regression guard. S02 already wired BuffKind::Buff classification and cleanse_debuffs() excludes it (see status_effect.rs:42, 197-209). This task only adds a new test file matching the DoD-mandated name. Zero src/ changes.

## Inputs

- `src/combat/status_effect.rs`
- `.gsd/milestones/M017/slices/S05/S05-RESEARCH.md`

## Expected Output

- `tests/status_blessed_cleanse_immune.rs`

## Verification

cargo test --test status_blessed_cleanse_immune
