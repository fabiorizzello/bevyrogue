---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T02: Make cue timeout force resume with structured state

Replace the current indefinite stuck-cue behavior with a bounded frame-budget timeout for windowed timeline barriers. The timeout must mark the barrier as timed out, persist enough structured context to inspect cast_id, skill_id, timeline, beat, cue, node/frame when known, request/resume the cue, and let combat continue without corrupting headless authority. Update stale R013 tests that currently document indefinite suspension.

## Inputs

- `tests/timeline/r013_failure_visibility.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`

## Expected Output

- `src/combat/runtime/cue_barrier.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/combat/turn_system/mod.rs`
- `tests/timeline/r013_failure_visibility.rs`
- `tests/timeline/timeline_cue_barrier_pipeline.rs`

## Verification

cargo test --test timeline r013_failure_visibility

## Observability Impact

Adds inspectable timeout result/message with graph/cue/node/frame context and proves timeout recovery via tests.
