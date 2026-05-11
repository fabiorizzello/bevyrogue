# S06: Tempo Resistance (100→50→25%) + Min Action Threshold — UAT

**Milestone:** M011
**Written:** 2026-04-28T08:46:11.853Z

# S06: Tempo Resistance (100→50→25%) + Min Action Threshold — UAT

**Milestone:** M011
**Written:** 2026-04-28

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: The slice has no UI and no live-runtime requirement. The 14-test integration suite in `tests/tempo_resistance.rs` directly exercises the full pipeline from unit spawn to CombatEvent bus to AV mutation, providing complete artifact-driven coverage. Human UAT is not required per the slice plan.

## Preconditions

- `cargo test` passes with zero failures in the working directory
- `src/combat/resistance.rs`, `src/combat/av.rs`, and `tests/tempo_resistance.rs` exist

## Smoke Test

Run `cargo test --test tempo_resistance` — all 14 tests must pass in under 2 seconds.

## Test Cases

### 1. TempoResistance multiplier curve

1. Open `tests/tempo_resistance.rs`, locate `tempo_resistance_multiplier_curve`.
2. Run: `cargo test --test tempo_resistance tempo_resistance_multiplier_curve -- --nocapture`
3. **Expected:** Test passes; `TempoResistance::default()` starts at `hit_count=0`; `multiplier()` returns 1.0, 0.5, 0.25 for hits 0, 1, 2+ respectively.

### 2. Three consecutive Delay events show diminishing returns

1. Run: `cargo test --test tempo_resistance three_consecutive_delays_show_diminishing_returns -- --nocapture`
2. **Expected:** Test passes; printed output shows three AV changes where |change2| = |change1| × 0.5 and |change3| = |change1| × 0.25.

### 3. End-to-end boss scenario with resistance curve

1. Run: `cargo test --test tempo_resistance boss_scenario_three_slow_hits_show_resistance_curve -- --nocapture`
2. **Expected:** Test passes; three `CombatEvent::TurnAdvance` events at -20% AV each are processed by `apply_turn_advance_system`; AV changes follow 100%→50%→25% attenuation (e.g., -2000 → -1000 → -500).

### 4. MIN_ACTION_THRESHOLD_AV floor prevents infinite delay

1. Run: `cargo test --test tempo_resistance apply_av_change_clamps_to_min_action_threshold -- --nocapture`
2. **Expected:** Test passes; a unit at AV=0 receiving a large negative Delay cannot be pushed below -15000 AV regardless of the delay magnitude.

### 5. Boss spawn gets TempoResistance, ally does not

1. Run: `cargo test --test tempo_resistance boss_spawn_gets_tempo_resistance_component ally_spawn_has_no_tempo_resistance_component -- --nocapture`
2. **Expected:** Both tests pass; ECS query confirms boss entity has `TempoResistance` component; ally entity does not.

### 6. Canonical units.ron contains Devimon with tempo_resistant: true

1. Run: `cargo test --test tempo_resistance canonical_units_ron_contains_tempo_resistant_boss -- --nocapture`
2. **Expected:** Test passes; RON file parses without error and Devimon's `UnitDef` has `tempo_resistant = true`.

### 7. Advances bypass resistance (stack not incremented)

1. Run: `cargo test --test tempo_resistance system_applies_advance_without_touching_resistance_stack -- --nocapture`
2. **Expected:** Test passes; after a positive Advance event, `TempoResistance.hit_count` remains 0 and AV increases by the full amount (no attenuation, no floor clamp).

## Edge Cases

### AV cannot exceed MAX_AV via Advance

1. Run: `cargo test --test tempo_resistance apply_av_change_advance_does_not_exceed_max_av -- --nocapture`
2. **Expected:** Test passes; AV is capped at MAX_AV=10000 even when the advance would push it higher.

### MIN_ACTION_THRESHOLD applies even without TempoResistance

1. Run: `cargo test --test tempo_resistance apply_av_change_clamps_without_resistance_too -- --nocapture`
2. **Expected:** Test passes; the floor is not exclusive to boss units — any unit receiving a Delay event cannot have AV pushed below -MIN_ACTION_THRESHOLD_AV.

## Failure Signals

- Any test in `tests/tempo_resistance.rs` failing indicates regression in resistance logic or boss wiring
- `cargo test` reporting failures in `turn_system_av` or `validation_snapshot` indicates AV model regression
- If `apply_turn_advance_system` is not registered in headless.rs `.chain()`, all integration tests that exercise the event bus will fail silently (AV unchanged)

## Not Proven By This UAT

- Live CLI demo with actual Slow skill applied via combat_cli (deferred to S09 UAT)
- Resistance behavior across multiple encounters (state reset between fights not tested)
- Interaction between TempoResistance and Stun/Break mechanics (those are S07 scope)

## Notes for Tester

The resistance curve is stepwise (not linear) — each hit bucket is independent. A unit going from 0→2 hit_count in one call to `apply_av_change` (e.g., via batched events) would not see the intermediate 50% step; each discrete `CombatEvent::TurnAdvance` increments the counter by one. If you observe flat AV deltas (no attenuation) on repeated delays, first check that `apply_turn_advance_system` is in the system chain in `src/headless.rs` before `advance_turn_system`.
