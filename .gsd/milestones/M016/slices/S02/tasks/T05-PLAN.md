---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T05: Repair Dorumon runtime proof and registry target

Update the Dorumon runtime proof so it asserts only the canonical drained `PredatorLoopResolved` event stream after kernel updates, not the transient `OnKernelTransition` envelope. Add the missing `tests/digimon_signal_registry.rs` integration target to prove Dorumon envelope parsing, registry routing, unknown-owner rejection, and malformed payload rejection, and wire it into Cargo so the verification command can run.

## Inputs

- `tests/dorumon_predator_runtime.rs`
- `tests/dorumon_blueprint.rs`
- `src/combat/blueprints/mod.rs`
- `src/combat/blueprints/dorumon.rs`
- `src/data/skills_ron.rs`

## Expected Output

- `dorumon_predator_runtime test passes with canonical event assertions`
- `digimon_signal_registry test target exists and passes`
- `Cargo test target list includes digimon_signal_registry`

## Verification

cargo test --test dorumon_predator_runtime --no-fail-fast && cargo test --test digimon_signal_registry --no-fail-fast
