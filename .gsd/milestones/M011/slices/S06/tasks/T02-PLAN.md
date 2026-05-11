---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Implement Tempo Resistance and Minimum Action Threshold

With the AV system in place, this task implements the core features of the slice: making bosses resistant to repeated `Delay` effects and ensuring they cannot be delayed indefinitely. This involves creating a new `TempoResistance` component and adding logic to the `TurnAdvance` event handler. A `MIN_ACTION_THRESHOLD_AV` constant will be defined (e.g., 15000).

## Inputs

- `src/combat/av.rs`
- `src/combat/turn_system/mod.rs`

## Expected Output

- `src/combat/resistance.rs`
- `tests/tempo_resistance.rs`

## Verification

cargo test --test tempo_resistance -- --nocapture
