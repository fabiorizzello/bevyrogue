---
id: S04
parent: M019
milestone: M019
provides:
  - perhop-length-guard
requires:
  []
affects:
  []
key_files:
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution.rs
  - tests/perhop_guard.rs
key_decisions:
  - D001 — kernel emits OnActionFailed diagnostic and clamps loop rather than panicking on PerHop length mismatch; load-time validator unchanged
patterns_established:
  - Pre-loop guard pattern: check curve length before entering per-hop loop, emit diagnostic event once, clamp bound to available coefficients
observability_surfaces:
  - CombatEventKind::OnActionFailed { reason } — emitted once per length-mismatch occurrence; flows through existing CombatEvent bus and jsonl_logger automatically
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-14T09:37:16.228Z
blocker_discovered: false
---

# S04: DamageCurve::PerHop runtime length guard (chiude follow-up #3 M018)

**Pre-loop PerHop length guard in pipeline.rs emits OnActionFailed diagnostic and clamps the bounce loop to available coefficients without panicking.**

## What Happened

T01 inserted a runtime length-guard in the Bounce path of `src/combat/turn_system/pipeline.rs` that fires before the per-hop loop whenever the action's `damage_curve` is `DamageCurve::PerHop(v)` and `v.len() < hops_planned`. The guard emits `CombatEventKind::OnActionFailed { reason: "DamageCurve::PerHop length {n} < hops_planned {h}" }` exactly once, then clamps the loop bound to `v.len()` so the action still resolves the hops it actually has coefficients for. The existing defensive `.min()` clamp in `resolution.rs::compute_hop_damage` was retained as belt-and-suspenders; its misleading doc-comment (which claimed a debug-panic) was corrected. A new integration test `tests/perhop_guard.rs` constructs a `ResolvedAction` directly with `PerHop(vec![30, 20])` and `TargetShape::Bounce { hops: 3, .. }`, bypassing `validate_skill_book`, runs it through `apply_effects` (per MEM003), and asserts all four invariants: no panic, exactly one `OnActionFailed` diagnostic, exactly 2 `OnDamage` events with the correct coefficients, and no ghost third hop. The load-time validator in `skills_ron.rs` was intentionally left unchanged per D001 (kernel never panics on bad data; the diagnostic event surfaces the mismatch at runtime).

## Verification

cargo test --test perhop_guard: 1 test (perhop_length_guard_emits_diagnostic_and_clamps_loop) — PASSED. cargo test (full suite): all integration test files pass, 0 failures, 0 regressions on target_shape_bounce_chain.rs, dr_pipeline.rs, cleanse_effect.rs, heal_effect.rs, and all other suites. Both runs exit 0.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

none

## Known Limitations

The load-time validator in skills_ron.rs does not check PerHop coefficient count against hops_planned at skill load time; mismatch is caught only at runtime via the new guard. This is intentional per D001 and deferred to M021 (trait Skill + SkillCtx).

## Follow-ups

none — closes M018 follow-up #3; M019 milestone is now complete (all slices S01–S04 done)

## Files Created/Modified

- `src/combat/turn_system/pipeline.rs` — Added pre-loop PerHop length guard: emits OnActionFailed diagnostic and clamps loop bound
- `src/combat/resolution.rs` — Corrected doc-comment on compute_hop_damage .min() clamp (no longer claims debug panic)
- `tests/perhop_guard.rs` — New integration test asserting the four invariants on a short-PerHop ResolvedAction
