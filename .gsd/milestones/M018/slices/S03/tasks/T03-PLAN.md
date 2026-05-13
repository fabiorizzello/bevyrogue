---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Generic kernel hop loop: dispatch on selector + repeat + damage curve read from inflight action

Refactor the Bounce arm in src/combat/turn_system/pipeline.rs so the kernel carries zero per-skill bias. Read (hops, selector, repeat) from the inflight action's TargetShape and the damage curve from the Effect::Damage being applied. Hop 0 = primary; for hop k in 1..hops rebuild TargetableSnapshot from actors, call select_bounce_hop(selector, snapshot, already_hit, enemy_team, last_target_slot), break on None. Track last_target_slot for selectors that need it (NextSlotAlive, AdjLowest). already_hit insertion gated by repeat policy: NoRepeat inserts after each hop; AllowRepeat skips insertion (or always-empty set). Per-hop damage computation: Constant → base_damage; Falloff{pct} → base_damage * (100 - pct*k) / 100 floored at 1; PerHop(v) → v[k]. SP/ult/streak hoist (S02 D04) unchanged — paid once pre-loop regardless of truncation. Add integration test tests/target_shape_bounce_chain.rs with 4 cases: (1) LowestHpPct + NoRepeat + Constant full chain N=3 no KO; (2) NextSlotAlive + NoRepeat + Falloff(20%) with KO mid-chain → chain truncates or skips to next slot; (3) LowestHpPct + AllowRepeat + PerHop[30,15,5] → same target may be hit twice when still lowest HP% after first hop; (4) pool exhaustion truncates silently. Each case asserts on per-hop damage delta from OnDamageDealt to prove curve is honored.

## Inputs

- `select_bounce_hop dispatcher (T01)`
- `TargetShape::Bounce struct + DamageCurve (T02)`

## Expected Output

- `generic kernel Bounce arm reading selector/repeat/curve from action`
- `integration test covering 4 selector/repeat/curve combos`

## Verification

cargo test --test target_shape_bounce_chain --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak
