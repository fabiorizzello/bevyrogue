---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T01: Add PerHop runtime length guard with diagnostic event and integration test

Insert a runtime length-guard in the bounce path of src/combat/turn_system/pipeline.rs that fires before the per-hop loop when the inflight action's damage_curve is DamageCurve::PerHop(v) and v.len() < hops_planned. The guard emits CombatEventKind::OnActionFailed { reason: "DamageCurve::PerHop length {n} < hops_planned {h}" } exactly once and clamps the loop bound to v.len() so the action still resolves the hops it has coefficients for. The defensive `.min()` clamp in src/combat/resolution.rs::compute_hop_damage stays as belt-and-suspenders; fix its doc-comment which currently lies about a debug panic. Add tests/perhop_guard.rs that bypasses validate_skill_book by constructing ResolvedAction directly with PerHop(vec![30, 20]) and TargetShape::Bounce { hops: 3, .. }, runs it through the pipeline via apply_effects (MEM003 pattern), and asserts the four invariants in success criteria. Per D001 the kernel never panics; load-time validator in skills_ron.rs is unchanged.

## Inputs

- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/combat/events.rs`
- `src/data/skills_ron.rs`
- `tests/target_shape_bounce_chain.rs`
- `.gsd/milestones/M019/slices/S04/S04-RESEARCH.md`
- `.gsd/DECISIONS.md`

## Expected Output

- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `tests/perhop_guard.rs`

## Verification

cargo test --test perhop_guard && cargo test

## Observability Impact

Adds one diagnostic emission: CombatEventKind::OnActionFailed { reason: "DamageCurve::PerHop length {n} < hops_planned {h}" } in the bounce pre-loop. A future agent can inspect the JSONL stream (via jsonl_logger) to detect blueprint-emitter bugs producing mismatched PerHop curves; the failure is no longer silent.
