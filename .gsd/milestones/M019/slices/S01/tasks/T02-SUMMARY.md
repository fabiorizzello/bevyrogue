---
id: T02
parent: S01
milestone: M019
key_files:
  - src/combat/damage.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - src/combat/follow_up.rs
key_decisions:
  - DR applied as final multiplicative step (not additive subtraction as in T01's partial attempt)
  - dr_reduction_pct capped at 100 in breakdown for display, while sum_dr remains unclamped internally
  - follow_up.rs maintains its own local ResolveActorsQuery which needed DrBag added separately
duration: 
verification_result: passed
completed_at: 2026-05-14T08:19:33.198Z
blocker_discovered: false
---

# T02: Integrated DR into calculate_damage formula with multiplicative factor and added dr_reduction_pct to DamageBreakdown

**Integrated DR into calculate_damage formula with multiplicative factor and added dr_reduction_pct to DamageBreakdown**

## What Happened

Added `defender_dr: Option<&DrBag>` parameter to `calculate_damage` in `src/combat/damage.rs`. The DR formula applies as a final multiplicative step: `dr_mod = (1.0 - sum_dr(defender_dr)).max(0.0)`, and `final_damage = round(...all other mods... * dr_mod).max(0) as i32`. Added `dr_reduction_pct: i32` field to `DamageBreakdown` which captures `(sum_dr.min(1.0) * 100).round()` for reporting.

Updated all call sites: the two production paths in `resolution.rs` (both `apply_damage_only` and the inline damage path in `apply_effects`) now pass `defender_dr` through to `calculate_damage`. All test calls in `resolution.rs` (17 occurrences) were updated to pass `None` as the 14th arg.

Also discovered that T01 had partially updated `resolution.rs` with a subtractive DR approach (raw_amount - dr) and had added `DrBag` to the Bevy query tuple in `turn_system/mod.rs` and `follow_up.rs` — this caused cascading tuple-arity mismatches across `pipeline.rs` and `follow_up.rs`. Fixed all 14-element destructuring patterns to 15-element across `pipeline.rs` (5 patterns), `turn_system/mod.rs` (1 pattern), and `follow_up.rs` (1 iter_mut + local ResolveActorsQuery definition updated to include DrBag).

Added 3 new DR-specific unit tests: 30% DR reduces 100→70, 100% DR floors to 0, >100% DR still floors to 0 with dr_reduction_pct capped at 100.

## Verification

cargo check passed clean (warnings only). cargo test --lib "damage::tests" passed 25 tests (22 matrix tests + 3 new DR tests).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | Clean compile, warnings only | 1700ms |
| 2 | `cargo test --lib damage::tests` | 0 | 25/25 tests passed including 3 new DR tests | 1140ms |

## Deviations

T01 had left a partial subtractive DR implementation in resolution.rs (raw_amount - sum_dr). This was replaced with the multiplicative approach by passing defender_dr through to calculate_damage. Also had to fix cascading tuple-arity errors in pipeline.rs and follow_up.rs caused by T01's addition of DrBag to the Bevy query.

## Known Issues

None.

## Files Created/Modified

- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
