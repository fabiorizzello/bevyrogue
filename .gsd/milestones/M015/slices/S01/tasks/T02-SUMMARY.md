---
id: T02
parent: S01
milestone: M015
key_files:
  - docs/m015_failure_ledger.md
key_decisions:
  - Keep `battery_loop_resolution` removed and treat `tests/battery_loop_kernel.rs` as the canonical replacement coverage.
  - Classify stale Holy Support and Twin Core references as architecture-drift candidates rather than restoring deprecated API surface.
duration: 
verification_result: mixed
completed_at: 2026-05-08T10:55:59.708Z
blocker_discovered: false
---

# T02: Rebuilt the M015 failure ledger with current compile, runtime, docs, and CLI blockers.

**Rebuilt the M015 failure ledger with current compile, runtime, docs, and CLI blockers.**

## What Happened

I replaced `docs/m015_failure_ledger.md` with a fuller, evidence-backed inventory that groups the current M015 blockers into explicit categories: the resolved `battery_loop_resolution` manifest blocker, mechanical `SkillDef`/`UnitDef`/`RoundFlags` fixture drift, obsolete Holy Support API tests, stale Twin Core state assertions, runtime reds, the missing UI readiness docs artifact, and the CLI runtime gap. I re-ran the key checks after T01 to make sure the ledger reflects the current state rather than the earlier snapshot: `cargo test --test battery_loop_kernel` still passes as the replacement coverage, `cargo test --no-run` now fails on the current compile reds instead of the stale manifest target, and `cargo run --bin combat_cli` now reaches runtime and panics on Bevy message initialization. I also verified the ledger contains the required named signals and no table rows are left as unknown/TBD, while explicitly warning downstream slices not to resurrect the old Holy Support or Twin Core APIs.

## Verification

Verified the rebuilt ledger with a passing replacement coverage test, a failing no-run compile inventory that exposes the current blocker set, a failing CLI runtime smoke run that reproduces the Bevy panic, and a doc sanity check confirming the required blocker names are present and no table cells are left as unknown/TBD.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test battery_loop_kernel` | 0 | ✅ pass | 352ms |
| 2 | `cargo test --no-run` | 101 | ❌ fail | 1413ms |
| 3 | `cargo run --bin combat_cli` | 101 | ❌ fail | 215ms |
| 4 | `grep/sanity check on docs/m015_failure_ledger.md required signals and table cells` | 0 | ✅ pass | 22ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `docs/m015_failure_ledger.md`
