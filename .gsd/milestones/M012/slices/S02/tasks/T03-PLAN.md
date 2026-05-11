---
estimated_steps: 5
estimated_files: 8
skills_used:
  - tdd
  - test
  - verify-before-complete
---

# T03: Preserve TargetShape metadata and reject non-single shapes before mutation

Executor skills_used frontmatter expectation: `tdd`, `test`, `verify-before-complete`.

Why: `TargetShape` is currently parsed but discarded, causing `Row` skills to mutate exactly one selected target. R085 requires Row/AllEnemies to execute truthfully or be explicitly unavailable. S02 chooses explicit deferral/rejection as the smallest safe step before S03/S04 add full DSL/query targeting metadata.

Do:
1. Add `target_shape: TargetShape` (or equivalent shape metadata) to `ResolvedAction` in `src/combat/state.rs` and populate it from the first `Effect::Damage { target, .. }` in `resolve_action`; default no-damage utility/revive skills to `TargetShape::Single` unless the skill data says otherwise.
2. Add a reusable predicate/helper near resolution or targeting code, e.g. `target_shape_is_executable_now(shape)`, that returns true only for `Single` in S02 and names `UnimplementedTargetShape` for `Row`, `AllEnemies`, and `SelfOnly`. Keep reason wording aligned with `docs/skill_legality_contract.md` for S04/S06 migration.
3. In `step_declaration`, after resolving the action but before `OnActionDeclared`/mutation, emit/log an `OnActionFailed { reason: "UnimplementedTargetShape:<Shape>" }` (or equivalent stable string containing `UnimplementedTargetShape`) and return `None` for non-single shapes. Do not consume SP, energy, ultimate charge, HP, toughness, or action lifecycle events for rejected shapes.
4. Add `tests/target_shape_truthfulness.rs` proving a Row skill and an inline/canonical AllEnemies fixture fail before mutation with the `UnimplementedTargetShape` reason, and proving a Single skill still executes normally. Avoid fixtures under ignored paths; inline any synthetic skill book data in the test or use tracked `assets/data/skills.ron`.
5. Update existing tests or tracked canonical data only where they used Row skills as ordinary single-target fixtures. Prefer changing test fixtures to a true Single skill over adding skill-ID exceptions. Do not add CLI/windowed hardcoding.

Failure Modes:
- Dependency: action lifecycle event ordering. Rejected shapes should not emit a misleading declared/pre-app/applied/resolved lifecycle as if execution happened; existing R070 tests must remain clear.
- Dependency: canonical Row skills. Tests such as `follow_up_triggers` may need fixture updates because `heat_viper` is Row in `assets/data/skills.ron`.

Load Profile:
- Shared resources: skill book lookup and event bus.
- Per-operation cost: one shape match per declared action; trivial.
- 10x breakpoint: none expected.

Negative Tests:
- Malformed inputs: no-damage skill without a Damage effect should not panic and should keep existing utility/revive behavior.
- Error paths: Row/AllEnemies/SelfOnly reject before mutation and before resource spend; unknown skill still follows existing `None` behavior unless already handled elsewhere.
- Boundary conditions: Single shape continues to execute and emit ordinary lifecycle/core events.

## Inputs

- `src/data/skills_ron.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/follow_up_triggers.rs`
- `assets/data/skills.ron`
- `docs/skill_legality_contract.md`

## Expected Output

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/target_shape_truthfulness.rs`
- `tests/follow_up_triggers.rs`

## Verification

cargo test-dev --test target_shape_truthfulness --test follow_up_triggers --test pipeline_dispatch && cargo test-dev

## Observability Impact

Rejected non-single shapes must be visible through `CombatEventKind::OnActionFailed` and `ActionLog` with an `UnimplementedTargetShape` reason so later query/engine-validation slices can assert parity.
