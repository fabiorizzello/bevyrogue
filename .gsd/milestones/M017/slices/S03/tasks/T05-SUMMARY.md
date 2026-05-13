---
id: T05
parent: S03
milestone: M017
key_files:
  - tests/status_amp_pipeline.rs
key_decisions:
  - advance_turn_system requires add_message::<ActionValueUpdated>() to be registered in test apps — its av_event_writer MessageWriter fails validation otherwise
  - StatusBag::apply used directly in test setup to pre-seed Heated/Chilled without routing through skill pipeline — simpler and still exercises the amp path
  - TurnAdvanced::of convenience constructor used for DoT test — AV metadata fields zeroed, sufficient for turn-end lifecycle
duration: 
verification_result: passed
completed_at: 2026-05-13T09:14:52.816Z
blocker_discovered: false
---

# T05: Integration test tests/status_amp_pipeline.rs: 4 cases cover fire/ice amp (100 vs 115) and Heated DoT (4 HP Fire) — all green

**Integration test tests/status_amp_pipeline.rs: 4 cases cover fire/ice amp (100 vs 115) and Heated DoT (4 HP Fire) — all green**

## What Happened

Created tests/status_amp_pipeline.rs with 4 deterministic headless tests:
A) fire_base100_non_heated_deals_100 — Vaccine vs Vaccine, Fire tag, no Heated → OnDamageDealt amount=100.
B) fire_base100_heated_defender_deals_115 — pre-applied Heated to defender via StatusBag::apply, same hit → amount=115.
C) ice_base100_chilled_defender_deals_115 — pre-applied Chilled to defender, Ice skill → amount=115.
D) heated_unit_turn_emits_dot_4_fire — spawned unit with Heated bag, wrote TurnAdvanced::of(UnitId(1)), asserted HP dropped by 4 and OnDamageDealt{amount:4, damage_tag:Fire, kind:Normal} appeared in event stream, source==target==UnitId(1).
Tests used advance_turn_system + resolve_action_system. Required adding add_message::<ActionValueUpdated>() to both app setups — advance_turn_system's av_event_writer parameter fails validation if this message type is not registered. Full cargo test: 0 failures.

## Verification

cargo test --test status_amp_pipeline: 4/4 pass. cargo test (full suite): 0 failures.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test status_amp_pipeline` | 0 | 4 passed, 0 failed | 8000ms |
| 2 | `cargo test` | 0 | full suite clean | 45000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/status_amp_pipeline.rs`
