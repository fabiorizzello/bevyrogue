---
id: T04
parent: S07
milestone: M002
key_files:
  - tests/digimon_kits/agumon_energy_gauge.rs
  - tests/digimon_kits.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - tests/digimon_kits/holy_support_resolution.rs
key_decisions:
  - Model the Agumon loop as a real runtime action sequence instead of a snapshot-only legality test: prove fill via repeated `ActionIntent::Basic`, prove lock with an actual failed `ActionIntent::Ultimate`, then prove drain with a successful ultimate cast.
duration: 
verification_result: passed
completed_at: 2026-05-21T21:00:22.551Z
blocker_discovered: false
---

# T04: Added a headless Agumon energy-loop integration test and fixed timeline-backed basics to actually fill the real energy gauge.

**Added a headless Agumon energy-loop integration test and fixed timeline-backed basics to actually fill the real energy gauge.**

## What Happened

Added `tests/digimon_kits/agumon_energy_gauge.rs`, a headless end-to-end regression that spawns real Agumon roster data plus a durable training dummy, then drives the combat runtime through the three required semantics: repeated `sharp_claws` casts fill `Energy.current`, an ultimate attempt fails with `UltimateNotReady` while energy is still below max even though legacy `UltimateCharge` is already primed, and a full-energy ultimate succeeds and drains both `Energy.current` and `UltimateCharge.current` to zero. To make the runtime honor the same semantics as legality, I patched `src/combat/turn_system/pipeline/timeline_exec.rs` so compiled-timeline actions also apply `energy_grant` and emit `EnergyGained` during finalize; otherwise Agumon's timeline-backed basics never mutated the real energy gauge and the integration loop could not reach readiness.

## Verification

Ran `cargo test --features windowed --test digimon_kits agumon_energy_gauge`; the new end-to-end regression passed and demonstrated fill, lock, and drain behavior through the real runtime.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test digimon_kits agumon_energy_gauge` | 0 | ✅ pass | 3306ms |

## Deviations

The planned integration test immediately exposed that compiled-timeline actions were not applying `energy_grant` at finalize time, so this task also patched `timeline_exec` to mirror the existing single-target/self-target/multi-target energy gain behavior before landing the regression test.

## Known Issues

None.

## Files Created/Modified

- `tests/digimon_kits/agumon_energy_gauge.rs`
- `tests/digimon_kits.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `tests/digimon_kits/holy_support_resolution.rs`
