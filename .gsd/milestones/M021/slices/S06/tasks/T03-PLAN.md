---
estimated_steps: 1
estimated_files: 8
skills_used: []
---

# T03: Remove effect-derived production dispatch and action state

Replace ResolvedAction fields and turn-pipeline wiring that exist only to shuttle legacy effect-derived data, keeping only metadata still required for action declaration, target bookkeeping, observability, and timeline dispatch. Delete the production timeline_backed branch and legacy apply_effects execution path from the active action flow, remove the Effect enum from the data model, eliminate helper code in src/combat/resolution.rs that scans effect lists to synthesize runtime behavior, and update runtime tests to prove timeline-only dispatch, bounce truthfulness, and revive/support semantics through the new single-path production flow.

## Inputs

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/tests.rs`
- `tests/compiled_timeline_runtime_dispatch.rs`
- `tests/target_shape_bounce_chain.rs`
- `tests/patamon_revive.rs`
- `assets/data/skills.ron`

## Expected Output

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/tests.rs`
- `tests/compiled_timeline_runtime_dispatch.rs`
- `tests/target_shape_bounce_chain.rs`
- `tests/patamon_revive.rs`

## Verification

cargo test --test compiled_timeline_runtime_dispatch --test target_shape_bounce_chain --test patamon_revive

## Observability Impact

Maintains the current combat-event inspection path after branch removal, with action lifecycle ordering and effect ordering still diagnosable from emitted events instead of hidden in legacy resolver state.
