---
id: T02
parent: S08
milestone: M012
key_files:
  - src/combat/action_query.rs — added `ActionAffordance`, enemy trait/charged telegraph affordance helpers, and snapshot imports
  - src/combat/turn_system/mod.rs — expanded the shared actor query tuple to carry `EnemyCounterplayKit`
  - src/combat/turn_system/pipeline.rs — updated actor/defender destructuring to the new snapshot shape
  - src/combat/follow_up.rs — updated the local resolve-actors query and follow-up destructuring to include counterplay
  - src/bin/combat_cli.rs — updated CLI query destructuring so the binary compiles against the expanded snapshot
  - src/combat/mod.rs — re-exported the new query helpers
  - tests/enemy_counterplay_affordance.rs — added contract tests for implemented, deferred, hidden, and empty counterplay declarations
  - tests/action_affordance_query.rs — fixed snapshot fixture literals for the expanded `UnitQuerySnapshot` shape
key_decisions:
  - Kept enemy counterplay as an explicit runtime component (`EnemyCounterplayKit`) rather than inferring it from toughness archetypes.
  - Reused the existing legality/status vocabulary (`ImplementationStatus`, `ResourceStatus`, `ResourceKind`, and deferred reason codes) so UI/CLI consumers can distinguish implemented, deferred, and hidden declarations without hardcoded skill IDs.
  - Re-exported the new query helpers from `combat::mod` to make the shared query surface easier for downstream consumers to use.
duration: 
verification_result: passed
completed_at: 2026-05-01T15:38:06.226Z
blocker_discovered: false
---

# T02: Propagated enemy counterplay declarations into runtime snapshots and query affordances

**Propagated enemy counterplay declarations into runtime snapshots and query affordances**

## What Happened

Extended the combat affordance layer so explicit enemy counterplay declarations survive spawn/snapshot translation and can be queried as structured implementation/resource statuses. Added the `EnemyCounterplayKit` runtime component path, wired `build_snapshot_from_ecs()` / `build_snapshot_from_ecs_with_sp()` to clone declaration data, and introduced pure affordance helpers for enemy traits and charged telegraphs. Re-exported the new query helpers from `combat::mod` for consumer convenience.

Built a dedicated contract test file covering the canonical Devimon projection, shielded Break Seal, empty minion declarations, hidden charged telegraphs, and the explicit regression that Armored toughness does not imply implemented Reactive Armor. The existing affordance suite kept passing after updating the snapshot tuple shape across `turn_system`, `follow_up`, and the CLI consumer paths that destructure the shared combat actor query.

## Verification

Ran the slice verification commands after the final code change: `cargo test-dev --test enemy_counterplay_affordance` and `cargo test-dev --test action_affordance_query`. Both targets passed, confirming the new counterplay affordances and the existing action affordance contract remained green.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test enemy_counterplay_affordance` | 0 | ✅ pass | 193ms |
| 2 | `cargo test-dev --test action_affordance_query` | 0 | ✅ pass | 161ms |

## Deviations

Patched additional local tuple destructures in `src/combat/follow_up.rs` and `src/bin/combat_cli.rs` because they used their own `Query` signatures rather than inheriting the turn-system alias, and both had to match the new 13-field snapshot shape.

## Known Issues

None.

## Files Created/Modified

- `src/combat/action_query.rs — added `ActionAffordance`, enemy trait/charged telegraph affordance helpers, and snapshot imports`
- `src/combat/turn_system/mod.rs — expanded the shared actor query tuple to carry `EnemyCounterplayKit``
- `src/combat/turn_system/pipeline.rs — updated actor/defender destructuring to the new snapshot shape`
- `src/combat/follow_up.rs — updated the local resolve-actors query and follow-up destructuring to include counterplay`
- `src/bin/combat_cli.rs — updated CLI query destructuring so the binary compiles against the expanded snapshot`
- `src/combat/mod.rs — re-exported the new query helpers`
- `tests/enemy_counterplay_affordance.rs — added contract tests for implemented, deferred, hidden, and empty counterplay declarations`
- `tests/action_affordance_query.rs — fixed snapshot fixture literals for the expanded `UnitQuerySnapshot` shape`
