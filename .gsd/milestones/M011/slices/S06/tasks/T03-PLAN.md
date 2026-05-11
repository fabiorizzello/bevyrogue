---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T03: Integrate and Verify Tempo Resistance on Boss Unit

This task integrates the new `TempoResistance` component with the game's data definitions (RON files) and verifies the entire system works end-to-end with a full scenario test. This ensures the component is correctly spawned on boss units and that the resistance mechanics function as expected in a simulated combat encounter.

## Inputs

- `src/combat/resistance.rs`
- `tests/tempo_resistance.rs`

## Expected Output

- `assets/data/units.ron`
- `src/data/units_ron.rs`

## Verification

cargo test --test tempo_resistance -- --nocapture
