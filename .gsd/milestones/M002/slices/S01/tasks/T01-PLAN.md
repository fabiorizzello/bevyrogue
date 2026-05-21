---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T01: Closed-enum schema extensions (AnimGraphId, FrameCue, ReleaseKernelCue, KernelCue predicate) + atomic asset/test migration

## Inputs

- None specified.

## Expected Output

- `src/animation/anim_graph.rs`
- `src/animation/mod.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/renamon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
- `tests/anim_graph_parse.rs`
- `tests/anim_validation.rs`

## Verification

cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation
