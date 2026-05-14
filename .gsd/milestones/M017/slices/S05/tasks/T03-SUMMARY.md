---
id: T03
parent: S05
milestone: M017
key_files:
  - src/combat/resolution.rs
  - tests/status_blessed_ult_charge.rs
key_decisions:
  - Blessed +1 Ult charge added after outcome.succeeded=true so both the Reset guard and the succeeded guard are coherent at that point.
  - Reset branch skipped per §H.1 to prevent the firing Ult from charging itself mid-cast.
duration: 
verification_result: passed
completed_at: 2026-05-13T09:56:19.364Z
blocker_discovered: false
---

# T03: Applied Blessed +1 Ult charge per action in apply_effects (skip on Reset branch); 3 integration tests confirm baseline, bonus, and no-leak on Ultimate cast

**Applied Blessed +1 Ult charge per action in apply_effects (skip on Reset branch); 3 integration tests confirm baseline, bonus, and no-leak on Ultimate cast**

## What Happened

In `src/combat/resolution.rs::apply_effects`, added a Blessed Ult-charge block after `outcome.succeeded = true`. The block fires only when `resolved.ult_effect != UltEffect::Reset` (§H.1 canon: skipping on Reset prevents the Ult from self-feeding its own meter) and when the attacker's StatusBag contains Blessed. When both conditions hold, `attacker_ult.try_add(1)` is called once. The `attacker_statuses: Option<&StatusBag>` parameter was already in place from T02 — no additional plumbing needed. Created `tests/status_blessed_ult_charge.rs` with three tests: (a) baseline Basic (no Blessed) → delta equals charge_per_event=25; (b) Blessed Basic → delta=26 (25+1); (c) Blessed Ultimate-cast (Reset branch, ult pre-set to ready) → meter resets to 0, Blessed does not leak a +1.

## Verification

cargo test --test status_blessed_ult_charge (3/3 pass), cargo test full suite (all test result: ok, 0 failed)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test status_blessed_ult_charge` | 0 | pass — 3 passed | 2800ms |
| 2 | `cargo test` | 0 | pass — all test result: ok, 0 failed | 8200ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/resolution.rs`
- `tests/status_blessed_ult_charge.rs`
