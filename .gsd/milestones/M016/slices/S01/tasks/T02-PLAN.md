---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Implement Tentomon blueprint logic

Create `src/combat/blueprints/tentomon.rs`. Implement `transitions_for_signal` matching `TentomonCustomSignal` to `CombatKernelTransition::BatteryLoop(...)` wrapping the respective `BatteryLoopTransition`. Update `src/combat/blueprints/mod.rs` to include the new module and dispatch `SkillCustomSignal::Tentomon` to it.

## Inputs

- `src/combat/blueprints/mod.rs`
- `src/combat/kernel.rs`
- `src/combat/battery_loop.rs`

## Expected Output

- `src/combat/blueprints/tentomon.rs`
- `src/combat/blueprints/mod.rs`

## Verification

cargo check && cargo test --no-run
