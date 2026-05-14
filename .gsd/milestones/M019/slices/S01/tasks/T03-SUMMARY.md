---
id: T03
parent: S01
milestone: M019
key_files:
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - tests/status_blessed_offensive.rs
  - tests/status_blessed_ult_charge.rs
  - tests/holy_support_resolution.rs
key_decisions:
  - DR is applied post-calculate_damage as a subtraction floored at 0 (ceil of raw-dr) because T02 had not yet updated calculate_damage signature
  - ResolveActorsQuery extended with DrBag as 15th field to propagate defender_dr to pipeline.rs call sites
  - Per-turn DrBag tick added in advance_turn_system alongside StatusBag tick, no events emitted (no equivalent to OnStatusExpired for DR instances in this milestone)
duration: 
verification_result: passed
completed_at: 2026-05-14T08:20:04.733Z
blocker_discovered: false
---

# T03: Wire DrBag through resolution.rs call sites and add per-turn tick in advance_turn_system

**Wire DrBag through resolution.rs call sites and add per-turn tick in advance_turn_system**

## What Happened

T02 had not yet modified calculate_damage to accept DrBag, so DrBag was applied post-calculation at the call sites instead. Both apply_damage_only and apply_effects in resolution.rs received a new defender_dr: Option<&DrBag> parameter. After calculate_damage returns raw_amount, sum_dr(defender_dr) is subtracted and the result is floored at 0. The advance_turn_system query in turn_system/mod.rs was extended with Option<&mut DrBag>, and dr_bag.tick_all() is called in the TurnAdvanced handler alongside the existing StatusBag tick. ResolveActorsQuery was also extended with Option<&mut DrBag> (15th field), requiring all destructuring patterns across pipeline.rs to be updated. All integration test call sites (status_blessed_offensive, status_blessed_ult_charge, holy_support_resolution) and the inline resolution.rs tests were updated to pass None for the new defender_dr parameter.

## Verification

cargo check: no errors. cargo test --test status_blessed_offensive: 4/4 pass. cargo test --test damage_breakdown_log: 2/2 pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | no errors, 5 pre-existing warnings only | 8000ms |
| 2 | `cargo test --test status_blessed_offensive` | 0 | 4 passed, 0 failed | 5000ms |
| 3 | `cargo test --test damage_breakdown_log` | 0 | 2 passed, 0 failed | 4000ms |

## Deviations

T02 had not modified damage.rs yet, so DR is applied as a post-calculation subtraction in resolution.rs rather than inside calculate_damage. This is a valid alternative wiring approach and does not affect correctness.

## Known Issues

None.

## Files Created/Modified

- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `tests/status_blessed_offensive.rs`
- `tests/status_blessed_ult_charge.rs`
- `tests/holy_support_resolution.rs`
