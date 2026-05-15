---
estimated_steps: 11
estimated_files: 8
skills_used: []
---

# T03: Remove effect-derived production dispatch and action state

Expected skills: `bevy`, `rust-best-practices`, `tdd`, `verify-before-complete`.

Why: after the shipped active roster is timeline-backed, the production turn pipeline and action model must stop deriving runtime behavior from scanned `Effect` lists, or the slice goal remains only half-done.

Do:
- Replace `ResolvedAction` fields and turn-pipeline wiring that exist only to shuttle legacy effect-derived data, keeping only the metadata still required for action declaration, target bookkeeping, observability, and timeline dispatch.
- Delete the production `timeline_backed` branch and legacy `apply_effects` execution path from the active action flow so active runtime behavior comes from compiled timelines only.
- Remove the `Effect` enum from the data model and eliminate helper code in `src/combat/resolution.rs` that scans effect lists to synthesize damage, revive, status, delay, or free-skill behavior for production execution.
- Update core runtime tests so they now prove timeline-only dispatch, bounce truthfulness, and revive or support semantics through the new single-path production flow.

Failure modes to guard:
- Boot must still fail before runtime if a migrated skill is malformed.
- Action lifecycle events must still emit in the same declared -> preapp -> applied -> resolved order after the branch disappears.

Done when: production active-skill execution no longer depends on `Effect`, the action state shape is simplified to timeline-era metadata, and the core runtime regression tests pass through the single dispatch path.

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

Maintains the current combat-event inspection path after the branch removal, with action lifecycle ordering and effect ordering still diagnosable from emitted events instead of hidden in legacy resolver state.
