---
id: T02
parent: S04
milestone: M011
key_files:
  - (none)
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-27T18:27:23.650Z
blocker_discovered: false
---

# T02: Integrate headless engine and implement real-time combat event logging in CLI.

**Integrate headless engine and implement real-time combat event logging in CLI.**

## What Happened

Integrated core combat systems and `DataPlugin` into `combat_cli`. Added a `bootstrap_system` that handles encounter initialization once data is ready. Implemented `event_logger_system` to stream `CombatEvent` kinds to the terminal. Added a TTY check to allow non-interactive verification while preserving the interactive `inquire` prompt for real users.

## Verification

Ran `cargo run --bin combat_cli`. Observed data loading, successful bootstrap, and `PartySelected`/`TurnOrderSeeded` events in the terminal output.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo run --bin combat_cli` | 0 | ✅ pass | 3000ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
