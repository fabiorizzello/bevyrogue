---
id: T01
parent: S04
milestone: M019
key_files:
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution.rs
  - tests/perhop_guard.rs
key_decisions:
  - D001: kernel never panics on PerHop length mismatch — emit diagnostic + truncate loop to available coefficients
duration: 
verification_result: passed
completed_at: 2026-05-14T09:35:38.476Z
blocker_discovered: false
---

# T01: Pre-loop PerHop length guard in pipeline.rs emits OnActionFailed and clamps the bounce loop; integration test passes.

**Pre-loop PerHop length guard in pipeline.rs emits OnActionFailed and clamps the bounce loop; integration test passes.**

## What Happened

The guard was inserted at the start of the bounce path in `src/combat/turn_system/pipeline.rs` (lines 812–833). Before the per-hop loop begins, the code checks whether `damage_curve` is `DamageCurve::PerHop(v)` and whether `v.len() < hops` (the planned hop count). On mismatch it emits exactly one `CombatEventKind::OnActionFailed { reason: "DamageCurve::PerHop length {n} < hops_planned {h}" }` and sets `clamped_hops = v.len()`, so the loop runs only for the coefficients that actually exist. The `compute_hop_damage` clamp in `resolution.rs` remains as belt-and-suspenders; its doc-comment was corrected to describe the actual clamp behaviour (no panic). `tests/perhop_guard.rs` was added, constructing a `SkillBook` with `PerHop(vec![30, 20])` and `hops: 3` without calling the load-time validator, running it through `resolve_action_system`, and asserting all four slice invariants: (a) no panic, (b) exactly one `OnActionFailed` naming the mismatch, (c) exactly two `OnDamageDealt` events, (d) amounts equal to `[30, 20]`.

## Verification

Ran `cargo test --test perhop_guard` (1 test, OK) and `cargo test` (full suite, all tests pass, 0 failures).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test perhop_guard` | 0 | pass | 160ms |
| 2 | `cargo test` | 0 | pass | 8000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `tests/perhop_guard.rs`
