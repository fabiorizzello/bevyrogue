---
id: T02
parent: S06
milestone: M011
key_files:
  - src/combat/resistance.rs
  - src/combat/turn_system/mod.rs
  - src/combat/mod.rs
  - src/headless.rs
  - tests/tempo_resistance.rs
key_decisions:
  - MIN_ACTION_THRESHOLD_AV=15000: allows up to 25000 AV needed to act after max delay, preventing infinite-lock while preserving meaningful Delay punishment
  - Resistance applies only to negative amount_pct (Delay); advances bypass resistance entirely
  - apply_turn_advance_system reads CombatEvent bus rather than inlining into pipeline.rs step_app — keeps pipeline simpler and follows single-source-of-truth event bus principle
  - f64 used for multiplier math to avoid integer truncation at 25% (e.g. -2000 * 0.25 = -500.0 rounds correctly)
duration: 
verification_result: passed
completed_at: 2026-04-28T07:02:54.853Z
blocker_discovered: false
---

# T02: Implemented TempoResistance component (100→50→25% curve) and MIN_ACTION_THRESHOLD_AV floor, wired to CombatEvent bus via apply_turn_advance_system

**Implemented TempoResistance component (100→50→25% curve) and MIN_ACTION_THRESHOLD_AV floor, wired to CombatEvent bus via apply_turn_advance_system**

## What Happened

With the AV system from T01 in place, this task adds the Tempo Resistance mechanic:

1. **src/combat/resistance.rs** (new): Defines `TempoResistance` component with `hit_count: u32` and `multiplier()` returning 1.0 → 0.5 → 0.25 for successive Delay hits. Also defines `MIN_ACTION_THRESHOLD_AV = 15_000` (floor: AV cannot drop below -15000, so a fully-delayed unit needs at most 25000 AV gain to act again). Pure functions `compute_av_change(amount_pct, resistance)` and `apply_av_change(av, resistance, amount_pct)` encapsulate all resistance + clamping logic, making them testable without a Bevy world.

2. **src/combat/turn_system/mod.rs**: Added `apply_turn_advance_system` which reads `MessageReader<CombatEvent>` and, for each `TurnAdvance` event, finds the target unit by `UnitId` and calls `apply_av_change`. Resistance stack is only incremented for negative (delay) amounts; positive advances are unaffected.

3. **src/combat/mod.rs**: Added `pub mod resistance;` to expose the new module.

4. **src/headless.rs**: Registered `apply_turn_advance_system` in the `.chain()` of combat systems, before `advance_turn_system`.

5. **tests/tempo_resistance.rs** (new): 10 tests split across pure-logic (8) and Bevy integration (2). Pure tests verify the resistance curve, `compute_av_change`, `apply_av_change`, MIN_ACTION_THRESHOLD_AV clamping, and the MAX_AV advance cap — all without a Bevy world. Integration tests confirm the system picks up `CombatEvent::TurnAdvance` from the message bus and correctly mutates `ActionValue` + `TempoResistance` on the target entity across two consecutive delay events.

## Verification

Ran `cargo test --test tempo_resistance -- --nocapture` — 10/10 pass. Ran full `cargo test` — 0 failures across all test crates.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test tempo_resistance -- --nocapture` | 0 | ✅ pass (10/10) | 560ms |
| 2 | `cargo test 2>&1 | grep -E 'FAILED|error\[' | wc -l` | 0 | ✅ pass (0 failures) | 3100ms |

## Deviations

None — task plan matches what was implemented exactly.

## Known Issues

None.

## Files Created/Modified

- `src/combat/resistance.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/mod.rs`
- `src/headless.rs`
- `tests/tempo_resistance.rs`
