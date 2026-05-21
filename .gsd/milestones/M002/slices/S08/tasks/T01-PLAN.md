---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T01: Add typed graph input purity contract

Introduce the smallest explicit graph-input/role seam needed for R009 and add animation-domain tests that prove roles are closed typed values rather than literals/world lookups. Keep AnimGraph data-only and deterministic: no mutable context object, no World parameter in graph/player evaluation, no ad-hoc Custom string escape hatch. Preserve current RON compatibility unless the tests prove a schema gap that must change.

## Inputs

- `.gsd/REQUIREMENTS.md`
- `src/animation/anim_graph.rs`
- `src/animation/player.rs`
- `tests/animation.rs`

## Expected Output

- `tests/animation/anim_graph_input_purity.rs`

## Verification

cargo test --test animation anim_graph_input_purity

## Observability Impact

Provides executable contract evidence for R009; no runtime logging expected.
