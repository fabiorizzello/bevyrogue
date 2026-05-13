---
id: T01
parent: S04
milestone: M017
key_files:
  - src/combat/turn_system/mod.rs
key_decisions:
  - Paralyzed ticks bag (duration decrements) unlike Stunned which drops status_opt without ticking — aligns with canon §H.1 always-skip semantics while preserving duration lifecycle.
  - status_opt binding made mut to allow tick_all() call inside Paralyzed block; Stunned block is unchanged.
duration: 
verification_result: passed
completed_at: 2026-05-13T09:23:53.131Z
blocker_discovered: false
---

# T01: Paralyzed always-skips action dispatch in process_turn_advanced_system; enemy-AI gate widened to exclude Paralyzed units.

**Paralyzed always-skips action dispatch in process_turn_advanced_system; enemy-AI gate widened to exclude Paralyzed units.**

## What Happened

Added `is_paralyzed: bool` to the local `Snap` struct (L389+), populated from the unit's `StatusBag` during snapshot collection by capturing `status_bag` in the query lambda and calling `b.has(&StatusEffectKind::Paralyzed)`. Added a Paralyzed skip block immediately after the existing Stunned block (order: Heated DoT → Stunned → Paralyzed → normal bag tick). The Paralyzed block: emits `OnStatusTick` for every active status (matching the normal tick path), calls `tick_all()` so duration decrements and `OnStatusExpired` fires for anything that expires, then emits `OnActionFailed { reason: "paralyzed" }`, drops borrows, and `continue`s — skipping action dispatch. Changed `status_opt` binding from immutable to `mut status_opt` to satisfy the mutable borrow inside the Paralyzed block. Widened the enemy-AI gate from `!snap.is_stunned` to `!snap.is_stunned && !snap.is_paralyzed`. No new event variants needed; `OnActionFailed` with reason string was already present.

## Verification

cargo check: 0 errors (12 pre-existing warnings). cargo test --lib turn_system: 4/4 passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 3500ms |
| 2 | `cargo test --lib turn_system` | 0 | 4 passed, 0 failed | 1500ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/turn_system/mod.rs`
