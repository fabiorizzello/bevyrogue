---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Process all matching graph events per batch

Replace the early return statements in the asset-event loop with continue (or restructure so each event is handled independently) so every AnimationGraph load in a batch populates its registry. Keep per-event error isolation: a bad graph warns and is skipped without aborting the loop.

## Inputs

- `src/animation/registry.rs`

## Expected Output

- `registry.rs loop continues past each event; T01 test passes`

## Verification

cargo test --test animation (T01 case now green); cargo test (full headless suite green)
