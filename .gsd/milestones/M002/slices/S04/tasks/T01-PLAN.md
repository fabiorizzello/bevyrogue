---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T01: Owner-neutral post-KO reaction seam wired

## Inputs

- None specified.

## Expected Output

- `src/combat/runtime/post_action.rs`
- `src/combat/runtime/registry.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/paths/single_target.rs`
- `tests/registry_internals.rs`

## Verification

cargo test --test unit_died_payload --test timeline_cue_barrier_pipeline
