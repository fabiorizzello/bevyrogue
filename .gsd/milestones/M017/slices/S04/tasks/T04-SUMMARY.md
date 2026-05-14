---
id: T04
parent: S04
milestone: M017
key_files:
  - tests/status_slowed_delay.rs
key_decisions:
  - Chained resolve_action_system → apply_turn_advance_system in test to match production schedule — AV update visible in same app.update() call.
  - Used get_cursor_current() (not get_cursor()) so first-apply and re-apply event windows are isolated — same pattern as status_paralyzed_skip.rs.
  - Pure-ApplyStatus{Slowed} skill with sp_cost=0 and no damage; outcome.succeeded=true from resolution.rs flow, so status application and TurnAdvance emission proceed normally.
duration: 
verification_result: passed
completed_at: 2026-05-13T09:35:25.228Z
blocker_discovered: false
---

# T04: Integration test status_slowed_delay.rs: first-apply emits TurnAdvance{−30} and pushes defender AV from 5000→2000; re-apply emits zero TurnAdvance events.

**Integration test status_slowed_delay.rs: first-apply emits TurnAdvance{−30} and pushes defender AV from 5000→2000; re-apply emits zero TurnAdvance events.**

## What Happened

Created tests/status_slowed_delay.rs. Spawned attacker (Vaccine ally, UnitSkills pointing to pure-ApplyStatus{Slowed,3} SkillDef) and defender (Vaccine enemy, ActionValue=5000, no TempoResistance, StatusBag). Chained resolve_action_system → apply_turn_advance_system to match production ordering (headless.rs). Used CombatRng::from_seed(0) to ensure accuracy roll passes (Vaccine vs Vaccine → 100% threshold). Cursor set to get_cursor_current() before first update; read only frame-new events per iteration. First apply: confirmed exactly one TurnAdvance{target=defender_id, amount_pct:-30} emitted, defender AV reduced to 2000 (5000−3000, full-strength no-resistance −30% of MAX_AV=10000), and OnStatusApplied precedes TurnAdvance in stream. Second apply: confirmed zero TurnAdvance events (refresh_max_dur path only, is_first_apply_slowed=false because bag already has Slowed).

## Verification

cargo test --test status_slowed_delay — 1 test passed, 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test status_slowed_delay` | 0 | pass | 720ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/status_slowed_delay.rs`
