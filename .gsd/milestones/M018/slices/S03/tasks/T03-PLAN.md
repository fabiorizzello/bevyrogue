---
estimated_steps: 4
estimated_files: 2
skills_used: []
---

# T03: Extend pipeline multi-target arm with Bounce hop loop + add integration test

In `src/combat/turn_system/pipeline.rs` around the multi-target arm (currently line 182: `if matches!(inflight.action.target_shape, TargetShape::Blast | TargetShape::AllEnemies)`), widen the match to also cover `TargetShape::Bounce(_)`. Inside the arm, branch on shape:
- For `Blast | AllEnemies`: existing `resolve_targets()` + per-target loop unchanged.
- For `Bounce(n)`: implement hop loop. Hoist SP/ult/streak in Phase 1 exactly as the existing path does (resources spent once-per-cast even if chain truncates — see S02 D04). Seed `let mut already_hit: HashSet<UnitId> = HashSet::from([inflight.action.target])`. Hop 0 = primary; call `apply_damage_only` against the primary entity. For hop k in 1..n: rebuild `TargetableSnapshot` from `actors.iter()` (alive-only filter handled inside `next_bounce_hop`), call `next_bounce_hop(&snapshot, &already_hit, attacker_team.opposite())`. If `None`, break. Otherwise, insert id into `already_hit`, look up entity via the stable `actor_pairs` Vec, call `apply_damage_only` on it. Phase 3 attacker-side effects remain after the loop, unchanged. Use the existing `get_many_mut` / borrow-release pattern (rebuild snapshot after each mut borrow drops).
Add new integration test `tests/target_shape_bounce_chain.rs` (functional naming per CLAUDE.md) following the structure of `tests/target_shape_blast_spillover.rs`. Cases: (1) N=3 full chain no KO, asserts 3 distinct OnDamageDealt events with slot_index tie-break order on equal HP%; (2) N=3 with primary HP-engineered so hop-2 KOs target, chain still hits hop 3 on remaining alive enemy or truncates if none; (3) only-one-alive-enemy → 1 hop only; (4) N=3 primary is lowest-HP%, confirm hop 1 does NOT revisit primary.

## Inputs

- ``src/combat/turn_system/pipeline.rs` — multi-target arm line 182 (verified), actor_pairs collection line 187, snapshot build line 191`
- ``src/combat/resolution.rs` (T01/T02 outputs) — must contain `next_bounce_hop` and `apply_damage_only` (S02)`
- ``src/data/skills_ron.rs` (T01 output) — Bounce(u8) variant`
- ``tests/target_shape_blast_spillover.rs` — template for new integration test (3-enemy mock encounter pattern)`

## Expected Output

- ``src/combat/turn_system/pipeline.rs` — multi-target arm widened to include Bounce(_); hop loop with already_hit HashSet and per-hop snapshot rebuild`
- ``tests/target_shape_bounce_chain.rs` — new integration test with ≥4 cases`

## Verification

cargo test --test target_shape_bounce_chain --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak
