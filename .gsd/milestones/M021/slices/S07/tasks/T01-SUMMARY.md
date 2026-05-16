---
id: T01
parent: S07
milestone: M021
key_files:
  - src/combat/api/event_filter.rs
  - src/combat/api/signal.rs
  - src/combat/api/event_bridge.rs
  - src/combat/api/passive_runner.rs
  - src/combat/api/runner.rs
  - src/combat/api/mod.rs
  - tests/passive_event_filters.rs
  - tests/passive_kitsune_grace.rs
  - tests/blueprint_signal_dispatcher.rs
key_decisions:
  - Represent passive subscriptions as composable runtime filters rather than owner/name trigger pairs.
  - Bridge all combat events into `Signal::CombatEvent` while preserving the legacy `kernel/ult_used` blueprint signal.
  - Flush queued intents between passive steps and stop outer timelines when they cycle back to entry outside explicit loop bodies.
duration: 
verification_result: passed
completed_at: 2026-05-16T10:37:54.302Z
blocker_discovered: false
---

# T01: Added composite passive event filters and loop-safe passive dispatch with bridged combat envelopes

**Added composite passive event filters and loop-safe passive dispatch with bridged combat envelopes**

## What Happened

Introduced a new `event_filter` module that supports composite `Any`/`All`/`Not` filters, typed combat-envelope predicates, and custom signal predicates without baking Digimon-specific names into the shared API. Extended `Signal` with a bridged `CombatEvent` variant and updated the event bridge so every combat message becomes a passive envelope while `UltimateUsed` still emits the legacy `kernel/ult_used` blueprint signal. Reworked `PassiveRunner` to store filter subscriptions, drive timelines through `BeatRunner` step-by-step, flush queued intents between steps so state-gated reactions can settle correctly, and stop when a one-pass passive graph cycles back to its entry beat while preserving BeatRunner’s 256-hop loop breaker for explicit loop bodies. Added a focused integration test that proves composite matching, same-frame cascade, and loop-halting behavior, and updated the existing kitsune/blueprint signal tests to cover the widened signal surface.

## Verification

Verified with the target integration test and two adjacent regressions: `cargo test --test passive_event_filters`, `cargo test --test passive_kitsune_grace`, and `cargo test --test blueprint_signal_dispatcher` all passed on the current tree. The first test exercises composite matching, same-frame cascade, and the loop breaker; the latter two confirm the passive kitsune path still resolves deterministically and the blueprint signal dispatcher still round-trips and rejects unregistered signals as expected.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test passive_event_filters` | 0 | ✅ pass | 2641ms |
| 2 | `cargo test --test passive_kitsune_grace` | 0 | ✅ pass | 452ms |
| 3 | `cargo test --test blueprint_signal_dispatcher` | 0 | ✅ pass | 483ms |

## Deviations

Added BeatRunner cursor/loop accessors and a one-pass outer-cycle stop rule in PassiveRunner so state-gated passive timelines can settle instead of spinning on stale reads. Also broadened the passive signal bridge to carry combat envelopes in addition to legacy blueprint signals.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/event_filter.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/event_bridge.rs`
- `src/combat/api/passive_runner.rs`
- `src/combat/api/runner.rs`
- `src/combat/api/mod.rs`
- `tests/passive_event_filters.rs`
- `tests/passive_kitsune_grace.rs`
- `tests/blueprint_signal_dispatcher.rs`
