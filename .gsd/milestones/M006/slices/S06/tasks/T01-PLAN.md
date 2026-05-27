---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Reproduce the single-graph-per-batch starvation in a headless test

Write a failing headless test that queues two AnimationGraph asset-load events in a single update and asserts both registries populate. Confirm it fails against the current early-return behavior at registry.rs lines 276 and 279.

## Inputs

- `src/animation/registry.rs`

## Expected Output

- `A red headless test asserting all queued graph events populate their registries`

## Verification

cargo test --test animation -- registry (new case fails red, reproducing the bug)
