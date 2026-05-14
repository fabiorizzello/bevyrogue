---
id: T01
parent: S03
milestone: M017
key_files:
  - src/combat/status_effect.rs
key_decisions:
  - status_amp_pct returns i32 multiplier (100 or 115) not a float to stay consistent with integer arithmetic in damage pipeline
duration: 
verification_result: passed
completed_at: 2026-05-13T08:58:22.002Z
blocker_discovered: false
---

# T01: Added status_amp_pct(bag, tag) → 115 for Heated+Fire / Chilled+Ice, 100 otherwise, with 4 passing unit tests

**Added status_amp_pct(bag, tag) → 115 for Heated+Fire / Chilled+Ice, 100 otherwise, with 4 passing unit tests**

## What Happened

Read status_effect.rs and types.rs. Added `use crate::combat::types::DamageTag` import, then `status_amp_pct(bag: &StatusBag, tag: DamageTag) -> i32` pure function returning 115 when (Heated && Fire) or (Chilled && Ice), else 100. Added 4 unit tests in existing `#[cfg(test)] mod tests` block: no-status→100, Heated+Fire→115, Heated+Ice→100 (wrong tag), Chilled+Ice→115. No coupling to damage or turn pipelines — pure lookup as specified.

## Verification

cargo test combat::status_effect::tests::status_amp -- --nocapture: 4/4 passed. cargo check: Finished with no errors (pre-existing warnings only).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test combat::status_effect::tests::status_amp -- --nocapture` | 0 | 4 passed, 0 failed | 8200ms |
| 2 | `cargo check` | 0 | Finished dev profile, no errors | 1610ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/status_effect.rs`
