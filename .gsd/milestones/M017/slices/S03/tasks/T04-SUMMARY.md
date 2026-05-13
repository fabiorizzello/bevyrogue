---
id: T04
parent: S03
milestone: M017
key_files:
  - src/combat/status_effect.rs
  - src/combat/turn_system/mod.rs
key_decisions:
  - chilled_speed_delta uses integer division (base_speed / 5) matching −20% rounded toward zero, consistent with existing i32 arithmetic
  - StatusBag already present in query tuple at index 7 — no query schema change needed, just renamed the wildcard to status_bag_opt
  - as_deref() used on Option<Mut<StatusBag>> to get &StatusBag without consuming the mutable ref
duration: 
verification_result: passed
completed_at: 2026-05-13T09:12:06.473Z
blocker_discovered: false
---

# T04: Chilled −20% Speed delta wired at AV-gain site via derived-read helper chilled_speed_delta

**Chilled −20% Speed delta wired at AV-gain site via derived-read helper chilled_speed_delta**

## What Happened

Added `chilled_speed_delta(bag: &StatusBag, base_speed: i32) -> i32` to `src/combat/status_effect.rs` returning `-(base_speed / 5)` when Chilled is present, else 0. In `src/combat/turn_system/mod.rs`, captured the `StatusBag` option (index 7, already in query tuple) from `_` into `status_bag_opt` in the AV-gain for-loop. Computed `chilled_delta` via `as_deref().map(...)` on the option, then added it to the speed sum before multiplying by `AV_PER_SPEED`. No `SpeedModifier` mutation — purely derived read as required by canon.

## Verification

cargo check — clean (warnings only, no errors). cargo test chilled — 3 new chilled_speed_delta tests pass plus 2 pre-existing chilled-related tests. cargo test --test combat_coherence — 3/3 pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 1700ms |
| 2 | `cargo test chilled` | 0 | pass — 5 tests (3 new + 2 pre-existing) | 3200ms |
| 3 | `cargo test --test combat_coherence` | 0 | pass — 3/3 | 1400ms |

## Deviations

none

## Known Issues

None.

## Files Created/Modified

- `src/combat/status_effect.rs`
- `src/combat/turn_system/mod.rs`
