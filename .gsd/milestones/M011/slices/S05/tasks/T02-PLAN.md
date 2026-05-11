---
estimated_steps: 18
estimated_files: 3
skills_used: []
---

# T02: Create Energy component and RoundEnergyTracker with per-turn gain caps

R073 requires per-unit Energy (max 100) with per-turn gain caps: 10 from secondary actions, 30 from external sources. No Energy component exists in the codebase. This task creates the module and wires it into mod.rs. Energy is not consumed by anything in this slice — S08 (Form Identity) will be the first consumer.

## Steps

1. Create `src/combat/energy.rs` with:
   - `Energy` component: `current: i32`, `max: i32` (default 100). Methods: `gain(amount: i32)`, `spend(amount: i32) -> bool`, `is_full() -> bool`.
   - `EnergyGainSource` enum: `SecondaryAction`, `External`.
   - `RoundEnergyTracker` component (per-unit, not resource): `secondary_gained: i32` (cap 10), `external_gained: i32` (cap 30). Method `try_gain(source: EnergyGainSource, amount: i32) -> i32` returns actual gain after cap. Method `reset()`.
2. Register module in `src/combat/mod.rs`: add `pub mod energy;`.
3. In `src/combat/bootstrap.rs`: add `Energy::default()` and `RoundEnergyTracker::default()` to the spawn bundle in `spawn_unit_from_def`.
4. Add `#[cfg(test)] mod tests` in `energy.rs`: (a) secondary cap at 10, (b) external cap at 30, (c) caps are independent, (d) reset restores full budget, (e) Energy::gain clamps at max.
5. Run `cargo test` — all tests pass.

## Must-Haves

- [ ] Energy component with max 100
- [ ] RoundEnergyTracker enforces 10/30 caps
- [ ] Module registered in mod.rs
- [ ] Units spawned with Energy and tracker

## Verification

- `cargo test` passes
- `grep -q 'pub mod energy' src/combat/mod.rs`

## Inputs

- ``src/combat/mod.rs``
- ``src/combat/bootstrap.rs``

## Expected Output

- ``src/combat/energy.rs` — Energy component + RoundEnergyTracker`
- ``src/combat/mod.rs` — energy module registered`
- ``src/combat/bootstrap.rs` — Energy + tracker in spawn bundle`

## Verification

cargo test 2>&1 | tail -5
