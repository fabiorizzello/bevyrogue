---
id: T03
parent: S03
milestone: M018
key_files:
  - src/combat/state.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - tests/target_shape_bounce_chain.rs
key_decisions:
  - DamageCurve stored on ResolvedAction (not re-read from skill book at hop time) — keeps kernel zero-bias from skill data at execution time while avoiding re-lookup
  - Snapshot rebuilt each hop inside the Bounce loop so KOs are reflected in selector candidates
  - NoRepeat inserts into already_hit after each hop; AllowRepeat never inserts (policy-driven exclusion, not a flag on select_bounce_hop itself)
  - Pool exhaustion (None from select_bounce_hop) breaks loop silently — no OnActionFailed emitted
  - SP/ult/streak hoisted once before the hop loop, consistent with S02 D04 design intent
duration: 
verification_result: passed
completed_at: 2026-05-13T21:46:37.241Z
blocker_discovered: false
---

# T03: Generic kernel Bounce hop loop with per-selector dispatch, repeat policy, and DamageCurve scaling read from inflight action

**Generic kernel Bounce hop loop with per-selector dispatch, repeat policy, and DamageCurve scaling read from inflight action**

## What Happened

T01 and T02 were already complete when T03 started: BounceSelector, RepeatPolicy, DamageCurve, and select_bounce_hop were all present in src/data/skills_ron.rs and src/combat/resolution.rs.

T03 added three things:

1. **DamageCurve on ResolvedAction** — Added `damage_curve: DamageCurve` to `ResolvedAction` in `src/combat/state.rs` and populated it from `skill_damage_curve()` in `resolve_action()` (resolution.rs). This lets the pipeline read the curve from the inflight action without touching the skill book again.

2. **Curve helpers in resolution.rs** — Added two public functions: `skill_damage_curve(effects)` (extracts DamageCurve from first Effect::Damage) and `compute_hop_damage(base_damage, curve, hop_k)` (Constant returns base; Falloff applies pct/100 multiplicatively per hop, floored at 1; PerHop indexes v[k]).

3. **Bounce execution path in pipeline.rs** — Added a `// === BOUNCE PATH ===` block in `step_app` that activates on `TargetShape::Bounce { hops, selector, repeat }`. The path: (a) hoists SP/ult/streak consumption once pre-loop, (b) loops `hop_k in 0..hops`, rebuilding the TargetableSnapshot each hop so KOs from prior hops are reflected, (c) calls `select_bounce_hop` with the `already_hit` set gated by `RepeatPolicy` (NoRepeat inserts after each hop; AllowRepeat skips insertion), (d) builds a per-hop `ResolvedAction` with `base_damage` overridden by `compute_hop_damage`, and (e) calls `apply_damage_only`. Pool exhaustion (select_bounce_hop returns None) breaks silently. Post-loop attacker resource effects (ult charge, SP gain, streaks, energy) mirror the Blast/AllEnemies path.

Updated 9 test files that directly constructed `ResolvedAction` literals to add `damage_curve: Default::default()`.

Integration test `tests/target_shape_bounce_chain.rs` covers all 4 required cases with `OnDamageDealt` amount and target assertions.

## Verification

cargo test --test target_shape_bounce_chain --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak; full cargo test suite (zero failures)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test target_shape_bounce_chain --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak` | 0 | 4 bounce + 2 blast + 2 aoe + 2 tiebreak = 10 tests passed | 5200ms |
| 2 | `cargo test` | 0 | Full suite passes with zero failures | 45000ms |

## Deviations

None. T01's select_bounce_hop was already present. T02's TargetShape::Bounce struct + DamageCurve were already present. Implemented as specified.

## Known Issues

None.

## Files Created/Modified

- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/target_shape_bounce_chain.rs`
