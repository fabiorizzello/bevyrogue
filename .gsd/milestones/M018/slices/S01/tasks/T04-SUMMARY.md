---
id: T04
parent: S01
milestone: M018
key_files:
  - tests/turn_advance_split.rs
key_decisions:
  - Tests use full ActionIntent→resolve_action_system→apply_av_ops_system pipeline (not direct apply_advance/apply_delay calls) to cover both emission-site cap and AV application in one shot; direct unit tests already exist in src/combat/resistance.rs
duration: 
verification_result: passed
completed_at: 2026-05-13T15:49:00.596Z
blocker_discovered: false
---

# T04: Added tests/turn_advance_split.rs with 6 deterministic boundary cases covering cap/clamp/floor/ceiling/TempoResistance/event-pct invariants for AdvanceTurn/DelayTurn

**Added tests/turn_advance_split.rs with 6 deterministic boundary cases covering cap/clamp/floor/ceiling/TempoResistance/event-pct invariants for AdvanceTurn/DelayTurn**

## What Happened

Created tests/turn_advance_split.rs from scratch. Six test functions cover all plan cases: (a) DelayTurn(80) capped at emission to 50 → AV MAX_AV→5000; (b) AdvanceTurn(80) capped to 50 → AV 0→5000; (c)+(d) double AdvanceTurn(50) from AV=10000 hits 2*MAX_AV ceiling, third call stays pinned; (e) DelayTurn(50) on AV=2000 clamps to floor 0; (f) DelayTurn(50) with TempoResistance{hit_count:2} (0.25 multiplier) → AV 10000→8750; (g) any AdvanceTurn/DelayTurn event in bus carries amount_pct ≤ 50. Used Messages::get_cursor_current() pattern from status_slowed_delay.rs. Shared build_app() helper parameterises initial AV and optional TempoResistance; fire_skill() drives ActionIntent → resolve_action_system → apply_av_ops_system pipeline. All 6 pass in 0.00s.

## Verification

cargo test --test turn_advance_split — 6/6 green, deterministic (no RNG/wall-clock). All plan boundary cases covered.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test turn_advance_split` | 0 | pass — 6 passed, 0 failed | 740ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/turn_advance_split.rs`
