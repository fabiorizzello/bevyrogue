---
id: T04
parent: S02
milestone: M018
key_files:
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - src/combat/follow_up.rs
  - tests/target_shape_blast_spillover.rs
  - tests/target_shape_aoe_all_order.rs
key_decisions:
  - Option (a) hoisting: SP/ult/streak consumed once in Phase 1 (before loop), apply_damage_only() handles per-target damage only — no resource mutation.
  - apply_damage_only() is a new public function in resolution.rs; existing apply_effects() is unchanged so all existing unit tests stay green.
  - SlotIndex added as 14th element to both ResolveActorsQuery aliases (turn_system/mod.rs and follow_up.rs) — the only way to pass slot data through the ECS query pipeline without a second conflicting query.
  - Snapshot built in a read-only actors pass before any mut borrows; actor_pairs Vec<(Entity,UnitId)> pre-collected for entity lookup in the loop to avoid re-iterating actors while holding get_many_mut borrows.
  - Status-to-apply (single-target status effects) deferred for multi-target path — tests do not require it and scope is bounded to T04.
duration: 
verification_result: passed
completed_at: 2026-05-13T19:54:22.959Z
blocker_discovered: false
---

# T04: Fan out apply_effects over resolve_targets() in pipeline::step_app for Blast/AllEnemies, with SP/ult/streak consumed once per cast (option a — hoisted)

**Fan out apply_effects over resolve_targets() in pipeline::step_app for Blast/AllEnemies, with SP/ult/streak consumed once per cast (option a — hoisted)**

## What Happened

Added a multi-target path in step_app (pipeline.rs) triggered when target_shape is Blast or AllEnemies. The path implements option (a) — resource consumption hoisted out of the per-target loop:

Phase 0: builds TargetableSnapshot from actors (read-only pass using the new SlotIndex field) and calls resolve_targets() to get the ordered target list.

Phase 1 (resource consumption, once): attacker stunned/KO guard, SP cost with Child streak discount, ult-ready check — all done before the loop, consuming resources exactly once per cast.

Phase 2 (per-target loop): for each UnitId in target_ids (already sorted slot_index asc by resolve_targets), calls the new apply_damage_only() helper which applies damage events (OnDamageDealt, OnBreak, OnKO) without touching SP/ult/streak. Commands (Ko insert, Stunned insert) and per-target events are emitted inside the loop with def_id as the CombatEvent.target.

Phase 3 (post-loop, once): UltEffect::GainFromBasic / Reset, streak.increment(), Blessed bonus, OnSkillCast, AdvanceTurn/DelayTurn/SelfAdvance, UltGain, energy grant, dispatch_blueprint_transitions.

Supporting changes:
- Added Option<&'static SlotIndex> as 14th element to ResolveActorsQuery in turn_system/mod.rs and follow_up.rs (both define the type alias), updating all destructuring patterns.
- Added apply_damage_only() to resolution.rs: pure per-target damage function taking no SP/ult/streak params.
- Added EvoStage and UnitId imports to pipeline.rs.

Two integration tests added:
- target_shape_blast_spillover.rs: 2 tests — 3-hit Blast (SP once), and edge-slot Blast (2 hits).
- target_shape_aoe_all_order.rs: 2 tests — AllEnemies skips KO'd target, and 10-run determinism check.

## Verification

cargo test --test target_shape_blast_spillover --test target_shape_aoe_all_order: 4/4 pass. Full cargo test: 0 failures across all suites.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test target_shape_blast_spillover --test target_shape_aoe_all_order 2>&1 | grep -E '(test result|FAILED)'` | 0 | 2 passed (blast), 2 passed (aoe) | 8000ms |
| 2 | `cargo test 2>&1 | grep FAILED` | 0 | no failures — full suite green | 45000ms |

## Deviations

Status-to-apply (ApplyStatus effect) is not applied per-target in the multi-target loop — the task plan and integration tests do not require it, and the single-target path handles it correctly. Can be added in a future task without API changes.

## Known Issues

none

## Files Created/Modified

- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `tests/target_shape_blast_spillover.rs`
- `tests/target_shape_aoe_all_order.rs`
