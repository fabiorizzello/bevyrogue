---
id: T02
parent: S05
milestone: M011
key_files:
  - src/combat/energy.rs
  - src/combat/mod.rs
  - src/combat/bootstrap.rs
key_decisions:
  - RoundEnergyTracker is a Component (per-unit) rather than a Resource — energy budgets are per-unit unlike SP which is party-shared; this matches the R073 design intent and avoids the global-resource pattern used by SpPool
duration: 
verification_result: passed
completed_at: 2026-04-27T20:02:43.065Z
blocker_discovered: false
---

# T02: Created Energy component and RoundEnergyTracker with per-turn gain caps (10 secondary / 30 external), wired into mod.rs and spawn bundle

**Created Energy component and RoundEnergyTracker with per-turn gain caps (10 secondary / 30 external), wired into mod.rs and spawn bundle**

## What Happened

No Energy component existed in the codebase. Created src/combat/energy.rs with three types: Energy (Component, current/max i32, default max=100, methods gain/spend/is_full), EnergyGainSource enum (SecondaryAction/External), and RoundEnergyTracker (Component, per-unit, secondary_gained cap 10, external_gained cap 30, methods try_gain/reset). RoundEnergyTracker is a Component (not a Resource like RoundSpTracker) since energy tracking is per-unit, not shared across the party. Registered the module in src/combat/mod.rs as `pub mod energy`. Updated bootstrap.rs to import Energy and RoundEnergyTracker and insert both into the spawn bundle inside spawn_unit_from_def. Added 5 inline unit tests covering: secondary cap at 10, external cap at 30, caps independent, reset restores full budget, Energy::gain clamps at max.

## Verification

cargo test — all test suites pass (131 unit tests + all integration suites, 0 failures). grep -q 'pub mod energy' src/combat/mod.rs returns exit 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test 2>&1 | grep -E 'test result'` | 0 | ✅ pass — all suites ok, 0 failures | 15000ms |
| 2 | `grep -q 'pub mod energy' src/combat/mod.rs && echo PASS` | 0 | ✅ pass | 50ms |

## Deviations

none

## Known Issues

none — Energy has no consumer yet; S08 (Form Identity) will be the first consumer of Energy::spend

## Files Created/Modified

- `src/combat/energy.rs`
- `src/combat/mod.rs`
- `src/combat/bootstrap.rs`
