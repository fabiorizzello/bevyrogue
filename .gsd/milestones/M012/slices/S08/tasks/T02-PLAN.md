---
estimated_steps: 6
estimated_files: 5
skills_used:
  - tdd
  - test
  - verify-before-complete
---

# T02: Propagate declarations into snapshots and query affordances

Why: The schema is not useful to UI/CLI unless it survives spawn and appears in the same shared query layer as action legality. This task adds the runtime component/snapshot seam and pure query helpers that distinguish implemented, deferred, and hidden enemy counterplay.

Failure Modes (Q5): ECS snapshot callers can silently drop declarations if their query tuple is not extended consistently; keep tuple additions explicit and update all call sites. Missing components should degrade to empty declaration vectors, not panics. Hidden/deferred declarations must preserve their own stable reason codes instead of inventing display strings.

Load Profile (Q6): Shared resources are ECS component queries and per-frame snapshot allocations. Per operation, declaration lists are short vectors cloned into `UnitQuerySnapshot`; at 10x combatants, repeated snapshot rebuilding is the first cost, so keep query helpers pure and consumers on one snapshot per frame/turn.

Negative Tests (Q7): Include empty declaration lists, hidden declarations, deferred charged telegraphs, implemented Tempo Anchor from declaration/runtime fact, and shielded-toughness Break Seal fixture. Include a regression that `Armored` toughness does not yield implemented Reactive Armor.

Add a lightweight component such as `EnemyCounterplayKit` (either in a new `src/combat/enemy_counterplay.rs` or an existing appropriate module) containing cloned typed `enemy_traits` and `charged_attack` declarations. Insert it in `bootstrap::spawn_unit_from_def()` for enemy/unit defs as appropriate and re-export the module if needed. Extend `UnitQuerySnapshot` with declaration fields and update the fallback constructor plus all test fixtures. Extend `build_snapshot_from_ecs()` / `build_snapshot_from_ecs_with_sp()` tuple inputs to accept `Option<&EnemyCounterplayKit>` and populate snapshot fields. Add pure query helpers in `src/combat/action_query.rs`, e.g. `query_enemy_trait_affordances(&UnitQuerySnapshot) -> Vec<EnemyTraitAffordance>` and `query_charged_telegraph_affordance(&UnitQuerySnapshot) -> Option<ChargedTelegraphAffordance>` or similarly named structs. Reuse existing `ImplementationStatus`, `ResourceStatus`, `ResourceKind::EnemyTrait`, `ResourceKind::ChargedTelegraph`, `LegalityReasonCode::EnemyTraitDeferred`, and `LegalityReasonCode::ChargedTelegraphDeferred`; do not implement Type Trap, Reactive Armor, or charged attack execution. Add `tests/enemy_counterplay_affordance.rs` with fixture-level contract tests for implemented Tempo Anchor, implemented Break Seal via `ToughnessCategory::Shielded`, deferred Type Trap/Reactive Armor, deferred/hidden charged telegraph, empty minion declarations, and canonical RON query projection.

## Inputs

- `src/data/units_ron.rs`
- `src/combat/action_query.rs`
- `src/combat/bootstrap.rs`
- `src/combat/mod.rs`
- `tests/action_affordance_query.rs`

## Expected Output

- `src/combat/action_query.rs`
- `src/combat/bootstrap.rs`
- `src/combat/mod.rs`
- `tests/enemy_counterplay_affordance.rs`

## Verification

cargo test-dev --test enemy_counterplay_affordance && cargo test-dev --test action_affordance_query
