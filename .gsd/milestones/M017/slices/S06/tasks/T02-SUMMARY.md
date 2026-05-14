---
id: T02
parent: S06
milestone: M017
key_files:
  - tests/status_observability_canon.rs
key_decisions:
  - Drain Messages<CombatEvent> cursor every frame inside the drive loop — Bevy 0.18 double-buffer clears oldest events each update(); batch drain after all frames loses events from early frames.
  - Anchor legacy/reserved check on "kind":"Burn" not raw "Burn" — damage_tag:"Fire" emitted by Heated DoT would false-positive on a bare string search.
  - Capture ValidationSnapshot between round 1 and round 2 so statuses are still active (dur=1) and the per-unit assertion is non-trivial.
duration: 
verification_result: passed
completed_at: 2026-05-13T11:04:15.101Z
blocker_discovered: false
---

# T02: New test tests/status_observability_canon.rs: 5 canon statuses on JSONL stream, zero legacy/reserved leakage, ValidationSnapshot.statuses deterministic per unit.

**New test tests/status_observability_canon.rs: 5 canon statuses on JSONL stream, zero legacy/reserved leakage, ValidationSnapshot.statuses deterministic per unit.**

## What Happened

Created tests/status_observability_canon.rs from scratch. Spawned 5 ally units (ids 1-5), each seeded with one canon status (Heated/Chilled/Paralyzed/Slowed/Blessed, dur=2) via StatusBag::apply. Registered only advance_turn_system; no skill pipeline needed since statuses are applied directly.

Drove 2 rounds × 5 units (10 app.update() calls). Critical gotcha: Bevy 0.18 Messages<T> uses a double-buffer that clears the oldest buffer on each update() — events older than 2 frames are gone. Draining event_cursor inside the per-frame loop (instead of after all frames) was required to capture all events.

After Round 1, captured ValidationSnapshot and asserted each unit carries its status with duration_remaining=1. After Round 2, all statuses expire (OnStatusExpired emitted for each kind).

Assertions: (a) "kind":"Heated", "kind":"Chilled", etc. each appear at least once in the joined stream; (b) "kind":"Freeze", "kind":"DeepFreeze", "kind":"Burn", "kind":"Shock" are absent — anchored on the "kind":"…" payload to avoid false-positives from damage_tag:"Fire" present in Heated DoT events; (c) per-unit snapshot matches expected ValidationStatusSnapshot vector.

## Verification

cargo check: clean. cargo test --test status_observability_canon: 1 passed. cargo test (full suite): 11 passed, 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 5000ms |
| 2 | `cargo test --test status_observability_canon` | 0 | pass — 1 test | 3000ms |
| 3 | `cargo test` | 0 | pass — 11 tests | 5000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/status_observability_canon.rs`
