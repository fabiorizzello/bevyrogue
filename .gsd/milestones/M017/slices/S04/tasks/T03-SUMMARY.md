---
id: T03
parent: S04
milestone: M017
key_files:
  - tests/status_paralyzed_skip.rs
key_decisions:
  - Use get_cursor_current() (not get_cursor()) for multi-frame message reading — get_cursor() at position 0 double-counts due to 2-frame buffer; get_cursor_current() records the write-head and each read() advances it.
  - OnActionFailed fires even on the last tick (duration 1→0) because is_paralyzed is captured from the pre-tick snapshot and tick_all() runs inside the Paralyzed block before the event fires — so 100 turns = exactly 100 skip events.
duration: 
verification_result: passed
completed_at: 2026-05-13T09:31:46.317Z
blocker_discovered: false
---

# T03: Integration test status_paralyzed_skip.rs: 100-turn Paralyzed loop asserts 100 skip events and zero ActionIntents from enemy.

**Integration test status_paralyzed_skip.rs: 100-turn Paralyzed loop asserts 100 skip events and zero ActionIntents from enemy.**

## What Happened

Created tests/status_paralyzed_skip.rs. Spawns 1 ally (UnitId 1) + 1 enemy (UnitId 2) with Paralyzed(dur=100) applied at construction. Loops 100 iterations: writes TurnAdvanced::of(UnitId(2)), calls app.update(), reads CombatEvents and ActionIntents via persistent MessageCursor initialized with get_cursor_current() before the loop. Key discovery: get_cursor() (position 0) double-counts due to the 2-frame message buffer — frame N's events persist into frame N+1. Using get_cursor_current() records the current write-head position, and each subsequent read() advances it, giving exactly that frame's events. Assertions: skip_count == 100 (one OnActionFailed{reason:"paralyzed"} per turn, including the tick where duration expires from 1→0) and enemy_intent_count == 0.

## Verification

cargo test --test status_paralyzed_skip: 1 passed, 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test status_paralyzed_skip` | 0 | 1 passed, 0 failed | 500ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/status_paralyzed_skip.rs`
