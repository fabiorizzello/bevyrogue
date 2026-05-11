# S06: Tempo Resistance (100→50→25%) + Min Action Threshold

**Goal:** Refactor the turn system to an Action Value (AV) model and implement Tempo Resistance for bosses and a Minimum Action Threshold for Delay effects.
**Demo:** scenario CLI con boss: 3 hit consecutivi di Slow mostrano resistenza crescente; test parametrizzato verifica la curva

## Must-Haves

- R078 (Tempo Resistance) is validated.
- The turn system is based on Action Value, not a `VecDeque`.
- Bosses show diminishing returns from repeated `Delay` effects.
- A unit's turn cannot be delayed past a defined `MIN_ACTION_THRESHOLD_AV`.
- All existing tests continue to pass after the turn system refactor.

## Proof Level

- This slice proves: - This slice proves: integration
- Real runtime required: no
- Human/UAT required: no

## Integration Closure

- Upstream: `src/combat/turn_system/mod.rs` and `src/combat/events.rs` are the main integration points. The concept of `TurnAdvance` is preserved, but its implementation is completely replaced.
- New Wiring: `ActionValue` component is added to all units. `advance_turn_system` is rewritten. `TempoResistance` is added to bosses. The system handling `TurnAdvance` events is modified to factor in resistance and the action threshold.
- What remains: Nothing for this mechanic. Downstream slices like S07 (Toughness) and S08 (Form Identity) can be built on this new, stable turn system.

## Verification

- Runtime signals: `CombatEvent` for `TurnAdvanced` will now contain the final `av_change` value, allowing logs to show the effect of resistance. A new `ActionValueUpdated` event could be added to trace AV changes for every unit after every turn.
- Inspection surfaces: The `ActionValue` component of each unit is directly inspectable in the ECS world, providing a clear view of the current turn order state.
- Failure visibility: If the turn order desyncs, the sorted list of entities and their AVs in the `TurnOrder` resource is the primary artifact to debug.

## Tasks

- [x] **T01: Refactor Turn System to Action Value (AV) Model** `est:3h`
  The current `VecDeque` turn system is incompatible with the design for `Delay` effects. An Action Value (AV) system is required to correctly implement scaling resistance and thresholds. This task performs the foundational refactoring of the turn system to an AV-based model as a prerequisite for the slice's main features. A design decision will be made and documented for the `MAX_AV` constant, likely 10000 based on HSR.
  - Files: `src/combat/turn_order.rs`, `src/combat/turn_system/mod.rs`, `src/combat/bootstrap.rs`, `src/combat/events.rs`, `src/combat/speed.rs`, `src/combat/mod.rs`
  - Verify: cargo test --test turn_system_av -- --nocapture

- [x] **T02: Implement Tempo Resistance and Minimum Action Threshold** `est:1h 30m`
  With the AV system in place, this task implements the core features of the slice: making bosses resistant to repeated `Delay` effects and ensuring they cannot be delayed indefinitely. This involves creating a new `TempoResistance` component and adding logic to the `TurnAdvance` event handler. A `MIN_ACTION_THRESHOLD_AV` constant will be defined (e.g., 15000).
  - Files: `src/combat/resistance.rs`, `src/combat/turn_system/mod.rs`, `src/combat/bootstrap.rs`, `src/combat/mod.rs`
  - Verify: cargo test --test tempo_resistance -- --nocapture

- [x] **T03: Integrate and Verify Tempo Resistance on Boss Unit** `est:45m`
  This task integrates the new `TempoResistance` component with the game's data definitions (RON files) and verifies the entire system works end-to-end with a full scenario test. This ensures the component is correctly spawned on boss units and that the resistance mechanics function as expected in a simulated combat encounter.
  - Files: `assets/data/units.ron`, `src/data/units_ron.rs`, `src/combat/bootstrap.rs`, `tests/tempo_resistance.rs`
  - Verify: cargo test --test tempo_resistance -- --nocapture

## Files Likely Touched

- src/combat/turn_order.rs
- src/combat/turn_system/mod.rs
- src/combat/bootstrap.rs
- src/combat/events.rs
- src/combat/speed.rs
- src/combat/mod.rs
- src/combat/resistance.rs
- assets/data/units.ron
- src/data/units_ron.rs
- tests/tempo_resistance.rs
