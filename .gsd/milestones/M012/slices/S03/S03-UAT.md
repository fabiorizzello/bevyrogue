# S03: S03: Skill DSL targeting and legality metadata — UAT

**Milestone:** M012
**Written:** 2026-04-30T22:05:08.562Z

# S03 UAT — Skill DSL targeting and legality metadata

## Preconditions
- Working tree contains the S03 migration of `SkillDef` metadata.
- Canonical skills are expected to be fully annotated in `assets/data/skills.ron`.
- No UI or CLI code should contain per-skill legality tables.

## Test Case 1 — Canonical catalog parses and validates
**Goal:** confirm the canonical data set now carries explicit targeting and implementation metadata and the loader accepts it.

**Steps**
1. Run `cargo test-dev skills_ron`.
2. Confirm the suite includes parse/round-trip and validation cases for targeting metadata.
3. Confirm the canonical catalog remains loadable.

**Expected outcomes**
- All `skills_ron` tests pass.
- Canonical skills parse without missing-field failures.
- Validation does not report contradictions for the shipped catalog.

## Test Case 2 — Canonical catalog contains explicit metadata for every skill
**Goal:** verify the migrated RON catalog is complete and uniform.

**Steps**
1. Run `grep -c "targeting:" assets/data/skills.ron`.
2. Run `grep -c "implementation:" assets/data/skills.ron`.

**Expected outcomes**
- `targeting:` count is `72`.
- `implementation:` count is `72`.
- Every canonical skill declares legality metadata instead of relying on inference.

## Test Case 3 — Malformed or contradictory metadata fails loudly
**Goal:** ensure invalid data is rejected at the correct layer.

**Steps**
1. Run the targeted `skills_ron` tests for malformed data and semantic contradictions.
2. Verify parse failures happen for missing/unknown targeting fields.
3. Verify semantic failures happen for contradicting declarations such as row damage marked as single, revive not targeting KO allies, and implemented non-single shapes.

**Expected outcomes**
- Missing required targeting metadata fails deserialization.
- Unknown targeting fields fail deserialization.
- Contradictory but well-typed skills fail validation with a skill id, category, and stable reason code.
- The reported reason codes are machine identifiers like `UnimplementedTargetShape` and `UnimplementedEffect`, not prose copy.

## Test Case 4 — Metadata drives resolved target shape
**Goal:** prove the runtime now trusts declared targeting metadata.

**Steps**
1. Run `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test patamon_revive`.
2. Inspect the target-shape regression cases.
3. Confirm the resolver still rejects non-single shapes before mutation and that revive semantics still work.

**Expected outcomes**
- `ResolvedAction.target_shape` reflects `SkillDef.targeting.shape`.
- Row/AllEnemies still fail before mutation with `UnimplementedTargetShape:<Shape>`.
- Revive and Patamon revive behavior remain green with explicit metadata.

## Test Case 5 — Windowed path still compiles
**Goal:** ensure the metadata changes did not break feature-gated UI compilation.

**Steps**
1. Run `cargo check --features "dev windowed"`.

**Expected outcomes**
- The windowed build compiles successfully.
- No feature-gated import or type regressions remain from the metadata migration.

## Edge Cases Covered
- `Damage(target: Row)` with `targeting.shape: Single` is rejected.
- `Revive` with non-KO targeting is rejected.
- Implemented non-single shapes are rejected.
- Mixed damage+revive skills remain deferred/hidden rather than pretending to be normal actions.

## UAT Result
S03 passes: the DSL now declares targeting/legalities explicitly, invalid inputs fail loudly, canonical data is complete, and runtime target-shape truth comes from metadata.
