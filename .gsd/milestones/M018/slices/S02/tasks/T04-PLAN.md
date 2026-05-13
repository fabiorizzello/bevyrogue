---
estimated_steps: 10
estimated_files: 4
skills_used: []
---

# T04: Fan out apply_effects in pipeline::step_app over resolve_targets()

In `src/combat/turn_system/pipeline.rs:160-292` (step_app), today: finds a single `target_entity` (~line 168) and calls `apply_effects` once (~line 278). Change: build the target list via `resolve_targets(&action.target_shape, action.target, &snapshot)`, then loop apply_effects over each defender.

**Critical constraint — resource consumption must fire exactly ONCE per cast, not per hit.** Current apply_effects mutates: attacker_ult (charge gain), sp_tracker (SP cost), basic_streak, and possibly energy. Pick one of:
  (a) Hoist resource consumption out of apply_effects into the caller (preferred — explicit, single responsibility), OR
  (b) Thread an `is_primary_hit: bool` flag through apply_effects and short-circuit consumption when false.

The planner picks (a) — hoist. Per-defender state stays inside the loop: defender_unit, defender_tough, defender_bag, OnDamageDealt event emission.

ResolvedAction stays unchanged (target = primary, target_shape unchanged) — multi-target is purely an apply-time concern; existing snapshot/log/event tests remain untouched.

Add two new integration tests:
- `tests/target_shape_blast_spillover.rs`: cast a Blast skill on a slot-1 primary in a 3-enemy formation. Assert 3 OnDamageDealt events emitted, slot_index 0,1,2 order, SP consumed once (not 3×).
- `tests/target_shape_aoe_all_order.rs`: cast an AllEnemies skill on a 3-enemy formation (one already KO'd). Assert OnDamageDealt fires only for the 2 alive enemies, in slot_index asc order. Run the test 10× via `for i in {1..10}; do cargo test --test target_shape_aoe_all_order -- --nocapture; done` to confirm determinism.

KO'd-adjacent Blast behavior: absorb. The resolver omits KO'd units, so step_app's loop naturally skips them — no special handling, no extra event.

## Inputs

- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/combat/sp.rs`
- `src/combat/ultimate.rs`

## Expected Output

- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `tests/target_shape_blast_spillover.rs`
- `tests/target_shape_aoe_all_order.rs`

## Verification

cargo test --test target_shape_blast_spillover --test target_shape_aoe_all_order 2>&1 | grep -E '(test result|FAILED)' && cargo test 2>&1 | tail -5
