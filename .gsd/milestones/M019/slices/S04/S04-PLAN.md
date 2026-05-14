# S04: DamageCurve::PerHop runtime length guard (chiude follow-up #3 M018)

**Goal:** Add a runtime length guard in the bounce pipeline so a dynamically constructed ResolvedAction with DamageCurve::PerHop(v) where v.len() < hops_planned does not panic, silently produce ghost hops, or mask the defect — emit a diagnostic CombatEvent and truncate the loop to v.len() so the action resolves cleanly with the coefficients it actually has.
**Demo:** Test tests/perhop_guard.rs: skill con DamageCurve::PerHop di lunghezza < hops_planned produce evento diagnostico (fail-fast o clamp — decisione registrata in DECISIONS.md) senza panic.

## Must-Haves

- tests/perhop_guard.rs constructs a bounce ResolvedAction with PerHop(vec![30, 20]) and hops=3, runs it through the pipeline, and asserts: (a) no panic, (b) exactly one CombatEventKind::OnActionFailed diagnostic emitted naming the length mismatch, (c) exactly 2 OnDamage events with the two coefficients applied, (d) no third "ghost" hop damage. cargo test full suite remains green (no regression on target_shape_bounce_chain.rs and other well-formed PerHop/Falloff tests). Decision D001 recorded.

## Proof Level

- This slice proves: integration — exercises the real bounce pipeline via apply_effects (no Bevy world spin-up), per MEM003.

## Integration Closure

Upstream surfaces consumed: src/combat/turn_system/pipeline.rs Bounce branch (lines 674-873), src/combat/resolution.rs compute_hop_damage (lines 309-334), src/combat/events.rs CombatEventKind::OnActionFailed. New wiring: pre-loop length check on PerHop curves in the Bounce path. Remaining end-to-end work after this slice: none for the M018 follow-up #3; milestone M019 closes after S04.

## Verification

- One new diagnostic emission path: CombatEventKind::OnActionFailed { reason: "DamageCurve::PerHop length {n} < hops_planned {h}" } emitted once pre-loop on length mismatch. Flows through the existing CombatEvent bus and jsonl_logger automatically. No new event variant; no exhaustiveness fallout in follow_up.rs / observability.rs / log.rs.

## Tasks

- [x] **T01: Add PerHop runtime length guard with diagnostic event and integration test** `est:1h`
  Insert a runtime length-guard in the bounce path of src/combat/turn_system/pipeline.rs that fires before the per-hop loop when the inflight action's damage_curve is DamageCurve::PerHop(v) and v.len() < hops_planned. The guard emits CombatEventKind::OnActionFailed { reason: "DamageCurve::PerHop length {n} < hops_planned {h}" } exactly once and clamps the loop bound to v.len() so the action still resolves the hops it has coefficients for. The defensive `.min()` clamp in src/combat/resolution.rs::compute_hop_damage stays as belt-and-suspenders; fix its doc-comment which currently lies about a debug panic. Add tests/perhop_guard.rs that bypasses validate_skill_book by constructing ResolvedAction directly with PerHop(vec![30, 20]) and TargetShape::Bounce { hops: 3, .. }, runs it through the pipeline via apply_effects (MEM003 pattern), and asserts the four invariants in success criteria. Per D001 the kernel never panics; load-time validator in skills_ron.rs is unchanged.
  - Files: `src/combat/turn_system/pipeline.rs`, `src/combat/resolution.rs`, `tests/perhop_guard.rs`
  - Verify: cargo test --test perhop_guard && cargo test

## Files Likely Touched

- src/combat/turn_system/pipeline.rs
- src/combat/resolution.rs
- tests/perhop_guard.rs
