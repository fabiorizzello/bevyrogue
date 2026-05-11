---
estimated_steps: 42
estimated_files: 4
skills_used: []
---

# T03: Source resolved target shape from DSL metadata

---
estimated_steps: 4
estimated_files: 4
skills_used:
  - tdd
  - verify-before-complete
---

Wire the existing resolution path to consume `SkillDef.targeting.shape` as the authoritative target-shape source and update regression tests so S02's unsupported-shape rejection will now be driven by S03 metadata instead of effect-shape inference.

Steps:
1. In `src/combat/resolution.rs`, change `resolve_action` so `ResolvedAction.target_shape` is copied from `skill.targeting.shape`. Remove or narrow the old `skill_target_shape(&skill.effects)` helper so no-damage skills no longer default to Single through effect inference.
2. Update `src/combat/resolution_tests.rs` to prove the new source of truth. Include a fixture where `Effect::Damage { target: Single }` but `targeting.shape: Row` resolves to Row, and a no-damage revive fixture that resolves to the explicit Single shape from metadata.
3. Update `tests/target_shape_truthfulness.rs` only as needed so Row/AllEnemies rejection still proves pre-mutation failure with `UnimplementedTargetShape:<Shape>` after the metadata migration.
4. Run revive and target-shape regressions to ensure existing execution behavior is preserved while shape truth now comes from metadata.

Must-haves:
- No code path infers selected target shape solely from `Effect::Damage` for `ResolvedAction`.
- S02 behavior remains true: non-single shapes are rejected before lifecycle mutation with stable `UnimplementedTargetShape:<Shape>` reasons.
- Revive and other no-damage skills use explicit metadata, not a fallback default, for selected shape.
- This task does not implement full side/life/self legality enforcement; that remains for S04/S06.

Failure Modes:
- **Metadata/effect mismatch**: validation from T02 should catch canonical contradictions; resolution should trust validated metadata and tests should make that boundary explicit.
- **Unsupported shape regression**: Row/AllEnemies must still fail before mutation, not execute as single-target.
- **Revive regression**: existing revive behavior must remain green; S03 only changes metadata source, not execution legality.

Load Profile:
- Shared resources: existing skill book lookup in resolution.
- Per-operation cost: reading a copied enum from `SkillDef` is trivial and cheaper than scanning effects.
- 10x breakpoint: no new scaling concern; resolution still performs the existing skill lookup.

Negative Tests:
- **Boundary conditions**: mismatched effect shape vs metadata shape fixture proves metadata wins; Row/AllEnemies tests prove unsupported shapes do not mutate state.
- **Error paths**: missing skill behavior remains `None` as before and is not widened in this task.

Verification:
- `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test patamon_revive`
- `cargo test-dev skills_ron`

Inputs:
- `src/combat/resolution.rs` — existing `resolve_action`, shape helper, and rejection helper.
- `src/combat/resolution_tests.rs` — direct resolution unit tests.
- `tests/target_shape_truthfulness.rs` — S02 integration tests for unsupported-shape rejection.
- `src/data/skills_ron.rs` — metadata schema and validation from T01/T02.

Expected Output:
- `src/combat/resolution.rs` — resolution copies target shape from `SkillDef.targeting.shape`.
- `src/combat/resolution_tests.rs` — tests prove metadata, not effect inference, drives `ResolvedAction.target_shape`.
- `tests/target_shape_truthfulness.rs` — regression tests updated for metadata fixtures if needed.
- `src/data/skills_ron.rs` — only touched if minor helper visibility adjustments are needed.

## Inputs

- `src/combat/resolution.rs`
- `src/combat/resolution_tests.rs`
- `tests/target_shape_truthfulness.rs`
- `src/data/skills_ron.rs`

## Expected Output

- `src/combat/resolution.rs`
- `src/combat/resolution_tests.rs`
- `tests/target_shape_truthfulness.rs`
- `src/data/skills_ron.rs`

## Verification

cargo test-dev --test target_shape_truthfulness --test revive_semantics --test patamon_revive && cargo test-dev skills_ron

## Observability Impact

Keeps S02's existing `OnActionFailed { reason: "UnimplementedTargetShape:<Shape>" }` failure surface intact, but ensures the shape in that diagnostic now comes from explicit DSL metadata.
