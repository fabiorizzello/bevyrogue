---
id: T02
parent: S05
milestone: M017
key_files:
  - src/combat/damage.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/damage_tests.rs
  - src/combat/resolution_tests.rs
  - tests/holy_support_resolution.rs
  - tests/status_blessed_offensive.rs
key_decisions:
  - attacker_dmg_mult is a plain f32 parameter on calculate_damage (not added to AttackContext) to keep AttackContext as a pure attack descriptor and avoid breaking the test helper atk().
  - Blessed mult computed in apply_effects (not calculate_damage) so the ×1.15 logic stays co-located with other status-driven damage modifiers.
  - Self-targeting pipeline path also receives attacker_bag so the signature is uniform; self-target actions have base_damage=0 so the mult never fires there.
duration: 
verification_result: passed
completed_at: 2026-05-13T09:51:41.268Z
blocker_discovered: false
---

# T02: Threaded attacker_dmg_mult through calculate_damage and applied Blessed ×1.15 in apply_effects; 4 integration tests confirm the bonus

**Threaded attacker_dmg_mult through calculate_damage and applied Blessed ×1.15 in apply_effects; 4 integration tests confirm the bonus**

## What Happened

Added `attacker_dmg_mult: f32` as a new last parameter to `calculate_damage` in `src/combat/damage.rs`, folded into the final product formula as `* attacker_dmg_mult`. Added `attacker_statuses: Option<&StatusBag>` to `apply_effects` in `src/combat/resolution.rs`; inside the damage path, computed `attacker_dmg_mult = 1.15 if bag.has(Blessed) else 1.0` and passed it to `calculate_damage`. Updated both `apply_effects` call sites in `pipeline.rs`: the self-targeting path captures `attacker_bag` from the query tuple (was `_`) and passes `attacker_bag.as_deref()`; the main action path similarly captures `attacker_bag` and passes it. Updated all 17 `apply_effects` calls in `resolution_tests.rs` and the one in `holy_support_resolution.rs` to pass `None` as the new `attacker_statuses` argument. Updated all 23 `calculate_damage` calls in `damage_tests.rs` to pass `1.0`. Created `tests/status_blessed_offensive.rs` with 4 assertions: Blessed → 115, no-bag → 100, empty-bag → 100, Heated-not-Blessed → 100. Full suite of 20+ test binaries all green.

## Verification

cargo check (no errors), cargo test --test status_blessed_offensive (4/4 pass), cargo test full suite (all test result: ok, 0 failed across all binaries)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 3200ms |
| 2 | `cargo test --test status_blessed_offensive` | 0 | pass — 4 passed | 2440ms |
| 3 | `cargo test` | 0 | pass — all test result: ok, 0 failed | 5100ms |

## Deviations

holy_support_resolution.rs also needed None added — not listed in the plan's Expected Output but required to compile.

## Known Issues

None.

## Files Created/Modified

- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/damage_tests.rs`
- `src/combat/resolution_tests.rs`
- `tests/holy_support_resolution.rs`
- `tests/status_blessed_offensive.rs`
