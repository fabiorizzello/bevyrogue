---
id: S08
parent: M012
milestone: M012
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["src/data/units_ron.rs", "assets/data/units.ron", "src/combat/action_query.rs", "src/combat/bootstrap.rs", "src/combat/mod.rs", "src/combat/turn_system/mod.rs", "src/combat/follow_up.rs", "src/bin/combat_cli.rs", "src/ui/combat_panel.rs", "tests/enemy_counterplay_affordance.rs", "tests/action_affordance_consumers.rs", "tests/roster_catalog.rs"]
key_decisions:
  - ["Typed EnemyCounterplayKind enum + serde-defaulted UnitDef fields instead of free-text signature_traits for UI-readiness", "ReactiveArmor is an explicit declaration only — never inferred from ToughnessCategory::Armored", "Reused existing ImplementationStatus/ResourceStatus/ResourceKind/LegalityReasonCode vocabulary so the query surface stays uniform", "Source-scan tests enforce the no-hardcoding contract in consumer files automatically"]
patterns_established:
  - ["Enemy counterplay declarations are explicit typed data in UnitDef/ECS, not derived from toughness or other orthogonal archetypes", "Expanding ECS actor query tuples requires updating every independent Query signature (follow_up.rs, combat_cli.rs are not covered by turn_system alias)", "Consumer display blocks branch only on enum variants from the shared query surface — no local legality logic in CLI or windowed panel"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-01T16:51:10.316Z
blocker_discovered: false
---

# S08: Enemy counterplay declaration surface

**Added typed enemy counterplay trait and charged-attack declarations from RON data through runtime ECS into pure query helpers, with consumer tests proving no hardcoded skill IDs or free-text trait names reach CLI or windowed UI.**

## What Happened

S08 closed the last unresolved R085 gap: enemy counterplay traits (TempoAnchor, TypeTrap, ReactiveArmor, BreakSeal) and charged-attack telegraphs now have a typed, queryable contract instead of relying on free-text `signature_traits` for UI decisions.

**T01 — Typed declarations in data/schema:** `EnemyCounterplayKind`, `EnemyCounterplayStatus`, `EnemyTraitDeclaration`, and `ChargedAttackDeclaration` were added to `src/data/units_ron.rs` with `#[serde(default)]` on `UnitDef.enemy_traits` and `UnitDef.charged_attack`, ensuring older RON fixtures deserialize without changes. The canonical roster was updated: Devimon declares TempoAnchor as Implemented plus TypeTrap/ReactiveArmor/charged-attack as Deferred; Ogremon carries a deferred charged-attack; Goblimon remains empty. The critical invariant — ReactiveArmor must not be inferred from `ToughnessCategory::Armored` — is enforced by a dedicated test in `roster_catalog.rs`.

**T02 — Runtime propagation and query surface:** The `EnemyCounterplayKit` ECS component was added and wired into `bootstrap::spawn_unit_from_def()`. `UnitQuerySnapshot` was extended with declaration fields and `build_snapshot_from_ecs()` / `build_snapshot_from_ecs_with_sp()` populate them. Pure query helpers `query_enemy_trait_affordances()` and `query_charged_telegraph_affordance()` were added to `action_query.rs`, reusing the existing `ImplementationStatus`, `ResourceStatus`, `ResourceKind`, and `LegalityReasonCode` vocabulary. The turn_system, follow_up, and CLI query tuples were all updated to carry the new component field. Contract tests in `tests/enemy_counterplay_affordance.rs` cover Tempo Anchor (Implemented), Break Seal via Shielded fixture, deferred Type Trap/Reactive Armor, hidden charged telegraph, empty minion, and the Armored→ReactiveArmor regression guard.

**T03 — Consumer exposure and hardcoding guards:** CLI and windowed panel consumers were extended with display blocks that read from the shared query surface only, branching on `ImplementationStatus` / `ResourceStatus` enum variants — never on enemy names, skill IDs, or `signature_traits` strings. Source-scan tests in `tests/action_affordance_consumers.rs` automatically enforce this contract for future contributors by grepping the consumer source files. Scenario TTK tests confirmed no behavioral regression from the new snapshot shape.

**Key patterns established:** Counterplay declarations are explicitly typed data, not inferred from toughness archetypes. The legality query vocabulary (`ImplementationStatus`, `ResourceKind`, `LegalityReasonCode`) is now the single source of truth for both action legality and enemy trait/telegraph affordances. Expanding ECS query tuples requires updating every system with its own Query signature independently (follow_up.rs and combat_cli.rs are not covered by the turn_system alias).

## Verification

All slice-level verification commands passed:
- `cargo test-dev --test enemy_counterplay_affordance` → 3/3 tests pass
- `cargo test-dev --test action_affordance_consumers` → 13/13 tests pass
- `cargo test-dev --test roster_catalog` → 2/2 tests pass
- `cargo test-dev` → full suite, all test targets pass, 0 failures
- `cargo check --features "dev windowed"` → Finished, no errors

## Requirements Advanced

- R085 — Enemy counterplay traits and charged-attack telegraphs are now typed queryable declarations in the shared affordance surface; UI consumers cannot falsely render them as usable because ImplementationStatus::Deferred/Hidden is the default for unimplemented mechanics

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

None.
