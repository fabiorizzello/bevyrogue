---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T04: Persist suspended timelines and resume only after cue release

## Inputs

- None specified.

## Expected Output

- `src/combat/runtime/cue_barrier.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/combat/turn_system/pipeline/mod.rs`
- `tests/timeline_cue_barrier_pipeline.rs`

## Verification

cargo test --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity --test compiled_timeline_runtime_dispatch
