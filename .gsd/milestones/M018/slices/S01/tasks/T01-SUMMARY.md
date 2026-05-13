---
id: T01
parent: S01
milestone: M018
key_files:
  - src/combat/av.rs
  - src/combat/resistance.rs
key_decisions:
  - Floor 0 replaces MIN_ACTION_THRESHOLD_AV for new primitives — infinite-delay lock prevention now structural (cap ±50 + TempoResistance curve + Speed accumulator)
  - Old apply_av_change/compute_av_change/MIN_ACTION_THRESHOLD_AV kept as shims until T02+ callers migrate
  - ActionValue::advance ceiling raised to 2*MAX_AV; is_ready() invariant unchanged at MAX_AV
duration: 
verification_result: passed
completed_at: 2026-05-13T15:13:54.936Z
blocker_discovered: false
---

# T01: Added apply_advance/apply_delay pure-logic functions with ±50% cap and [0, 2*MAX_AV] clamp; raised ActionValue::advance ceiling to 2*MAX_AV

**Added apply_advance/apply_delay pure-logic functions with ±50% cap and [0, 2*MAX_AV] clamp; raised ActionValue::advance ceiling to 2*MAX_AV**

## What Happened

Rewrote the AV math layer as pure functions (no Bevy, no event bus). Added `CAP_PCT = 50` constant and two new functions in `resistance.rs`: `apply_advance(av, pct)` (caps pct at 50, adds raw AV, clamps [0, 2*MAX_AV]) and `apply_delay(av, pct, resistance)` (caps pct at 50, applies TempoResistance curve delay-only, subtracts, clamps floor to 0). Both return the actual AV delta. Floor 0 replaces MIN_ACTION_THRESHOLD_AV for the new primitives — rationale documented in code (cap ±50 + TempoResistance curve + Speed accumulator = no infinite-delay lock). Old `compute_av_change`/`apply_av_change`/`MIN_ACTION_THRESHOLD_AV` retained as shims until T02+ callers migrate. In `av.rs`, changed `ActionValue::advance` and `self_advance` ceiling from `MAX_AV` to `2*MAX_AV` so over-advanced AV is representable; `is_ready()` invariant unchanged (>= MAX_AV). Added 8 inline `#[cfg(test)]` boundary tests covering: cap enforcement, 2x ceiling, multi-advance clamp, floor-0 delay, resistance curve, hit recording, and resistance bypass by advance.

## Verification

cargo check: clean (warnings only, no errors). cargo test --lib resistance: 8/8 new inline tests pass. cargo test --test tempo_resistance: 14/14 legacy integration tests pass — shims and existing event-bus path unchanged.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 4140ms |
| 2 | `cargo test --lib resistance` | 0 | 8/8 pass | 2660ms |
| 3 | `cargo test --test tempo_resistance` | 0 | 14/14 pass | 3000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/av.rs`
- `src/combat/resistance.rs`
