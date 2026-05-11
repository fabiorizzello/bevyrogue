---
estimated_steps: 5
estimated_files: 6
skills_used:
  - tdd
  - verify-before-complete
---

# T04: Add toughness affordances and close the public query contract

Why: S04 also supports R085 by exposing truthful affordances for deferred/hidden mechanics and enemy-only toughness. This task finishes the query contract, proves compatibility with established S02/S03 regressions, and runs the slice-level verification suite.

Skills: use `verify-before-complete` before claiming task and slice completion; use `tdd` if adding missing toughness tests first.

Do:
1. Add query-side toughness affordance output in `src/combat/action_query.rs` that uses `exposes_toughness_affordance` / `visible_toughness` from `src/combat/toughness.rs` rather than duplicating enemy-only logic.
2. Ensure enemy units with positive toughness expose visible implemented toughness data, while ally toughness and non-positive enemy bars are hidden/disabled with `ToughnessEnemyOnly` or no visible bar according to the public status vocabulary.
3. Add/complete tests in `tests/action_affordance_query.rs` covering enemy visible toughness, ally hidden/disabled toughness, hidden self-only form-identity-like skill, and deferred unsupported target shape in the final one-call action query.
4. Run focused S02/S03 regressions to confirm the new query vocabulary did not break canonical data, revive, target-shape truthfulness, or toughness enemy-only behavior.
5. Optionally run `cargo check --features "dev windowed"` if public type/export changes ripple into feature-gated imports; fix compile issues but do not wire UI consumption in S04.

Failure Modes (Q5): duplicated toughness logic could drift from S02 helpers; public export mistakes could compile in headless but break feature-gated consumers; hidden/deferred skills could be accidentally shown as disabled instead of hidden/deferred.

Load Profile (Q6): toughness affordance computation is per-unit and cheap; at 10x units the cost remains bounded by the same snapshot scan used for target affordances.

Negative Tests (Q7): ally toughness present internally but hidden from query, enemy zero/maxless toughness not shown as breakable, hidden skill status preserved, deferred row shape status preserved, and existing regression tests remain green.

## Inputs

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
- `src/combat/toughness.rs`
- `tests/toughness_enemy_only.rs`
- `tests/target_shape_truthfulness.rs`
- `tests/revive_semantics.rs`
- `tests/skill_legality_contract_docs.rs`

## Expected Output

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`

## Verification

cargo test-dev --test action_affordance_query && cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs && cargo test-dev

## Observability Impact

The query exposes toughness visibility and hidden/deferred implementation state directly, so future CLI/windowed adapters can display truthful affordances without inspecting internal `Toughness` components or skill IDs.
