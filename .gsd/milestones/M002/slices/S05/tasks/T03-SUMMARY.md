---
id: T03
parent: S05
milestone: M002
key_files: []
key_decisions: []
duration: 
verification_result: passed
completed_at: 2026-05-21T17:56:03.285Z
blocker_discovered: false
---

# T03: OnHitTaken drives frame-counted target blink/hurt projection

**OnHitTaken drives frame-counted target blink/hurt projection**

## What Happened

OnHitTaken CombatEvent triggers deterministic frame-window tint on the target sprite. Duration is frame-counted; no wall-clock dependency. UI path read-only.

## Verification

Blink/hurt projection visible; deterministic frame count; cargo test passes

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 0ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
