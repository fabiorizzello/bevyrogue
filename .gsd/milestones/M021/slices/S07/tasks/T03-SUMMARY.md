---
id: T03
parent: S07
milestone: M021
key_files:
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/blueprints/gabumon.rs
  - src/combat/blueprints/patamon/mod.rs
  - src/combat/blueprints/renamon.rs
  - src/combat/kernel.rs
  - src/combat/plugin.rs
  - tests/passive_canon_support.rs
key_decisions:
  - Boot canonical passive listeners from the combat plugin after core resources are initialized.
  - Use a shared `kernel/ult_used` passive trigger with per-blueprint guard keys and fixed canonical owners.
duration: 
verification_result: mixed
completed_at: 2026-05-16T11:11:05.772Z
blocker_discovered: false
---

# T03: Bootstrapped canonical Agumon, Gabumon, Patamon, and Renamon passive listeners into the combat runtime and added a shared passive integration test.

**Bootstrapped canonical Agumon, Gabumon, Patamon, and Renamon passive listeners into the combat runtime and added a shared passive integration test.**

## What Happened

Added passive-runtime helpers for Agumon, Gabumon, Patamon, and Renamon that register their passive timelines, shared `kernel/ult_used` filters, and blueprint-signal taxonomy entries through the combat runtime bootstrap. Wired the full combat plugin to install those canonical passive runners automatically at boot, while keeping the existing Twin Core and Holy Support kernel hooks/resources intact. Added a new canonical integration test that spawns the four Digimon plus a dummy ally, drives an `UltimateUsed` event through the passive dispatcher, and checks that the expected blueprint reactions are emitted and the SignalBus drains cleanly. Also added Renamon self/enemy negative coverage for Kitsune Grace.

## Verification

Verification was not completed before context budget compaction; the intended command is `cargo test --test passive_canon_support -- --nocapture`. Next step: run that test, fix any compile/runtime issues surfaced by the new passive bootstrap, and then re-run the targeted test until it passes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `Verification command planned but not executed to completion before compaction: `cargo test --test passive_canon_support -- --nocapture`` | -1 | unknown (coerced from string) | 0ms |

## Deviations

Used fixed canonical UnitId owners for the passive bootstrap helpers so the runtime can install the listeners automatically at boot without additional per-test setup.

## Known Issues

Compile/test status is unconfirmed because the targeted cargo test did not finish before compaction. The new passive test currently asserts blueprint reactions plus guard-state writes, but the runtime wiring still needs a full build/test pass to confirm there are no import/syntax issues or lifecycle regressions.

## Files Created/Modified

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/gabumon.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/renamon.rs`
- `src/combat/kernel.rs`
- `src/combat/plugin.rs`
- `tests/passive_canon_support.rs`
