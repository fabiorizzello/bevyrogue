---
id: T02
parent: S03
milestone: M017
key_files:
  - src/combat/damage.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/resolution_tests.rs
  - src/combat/damage_tests.rs
  - tests/holy_support_resolution.rs
key_decisions:
  - status_amp_pct kept as _status_amp_pct (prefixed underscore) in DamageBreakdown destructuring inside apply_effects — unused by OnDamageDealt event for now, retained for log/snapshot symmetry per slice plan
  - Self-targeting path in pipeline.rs passes None for defender_status — attacker == defender in that branch, no separate StatusBag ref available without query restructuring
  - holy_support_resolution.rs in tests/ was an undocumented call site; updated to None (regression-safe)
duration: 
verification_result: passed
completed_at: 2026-05-13T09:02:35.050Z
blocker_discovered: false
---

# T02: Wired status_amp_pct into calculate_damage and apply_effects: DamageBreakdown gains status_amp_pct field, pipeline passes defender StatusBag to apply_effects at both call sites

**Wired status_amp_pct into calculate_damage and apply_effects: DamageBreakdown gains status_amp_pct field, pipeline passes defender StatusBag to apply_effects at both call sites**

## What Happened

Extended DamageBreakdown with pub status_amp_pct: i32. Changed calculate_damage signature to accept defender_status: Option<&StatusBag> as 5th arg; applies amp_pct/100 as a fourth multiplicative factor (after tag_mod, tri_mod, break_mod). Added StatusBag import to damage.rs. Extended apply_effects in resolution.rs with defender_status: Option<&StatusBag> param forwarded to calculate_damage; destructuring updated with status_amp_pct: _status_amp_pct (kept in breakdown, unused by event for now per slice plan). Updated pipeline.rs self-targeting path (line ~280) to pass None (no separate defender bag in that branch) and the normal path (line ~576) to pass defender_bag.as_deref(). Updated all apply_effects callers in resolution_tests.rs to pass None. Discovered holy_support_resolution.rs in tests/ also called apply_effects with 11 args — added None there too. All damage_tests.rs calculate_damage calls updated to 5-arg form.

## Verification

cargo check → clean. cargo test → all 144+145+... suites pass, 0 failures across the full workspace.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 1240ms |
| 2 | `cargo test` | 0 | pass — all test suites ok, 0 failed | 4500ms |

## Deviations

Undocumented call site in tests/holy_support_resolution.rs also needed None added; not listed in task plan inputs but trivial fix.

## Known Issues

none

## Files Created/Modified

- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution_tests.rs`
- `src/combat/damage_tests.rs`
- `tests/holy_support_resolution.rs`
