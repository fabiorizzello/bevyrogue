# S04: Agumon/Gabumon Twin Core Refinement Blueprint

**Goal:** Migrate Twin Core mechanics for Agumon and Gabumon to their respective blueprints and data-driven custom signals.
**Demo:** After this: Twin Core mechanics operate through their respective blueprints.

## Must-Haves

- `agumon.rs` and `gabumon.rs` blueprints exist and are registered.
- `skills.ron` defines `custom_signals` for Agumon and Gabumon.
- Integration tests confirm custom signals are dispatched to valid `TwinCoreDesignTag` additions.

## Proof Level

- This slice proves: contract

## Integration Closure

Agumon and Gabumon combat logic will exclusively use blueprint-driven custom signals, decoupling the roster specifics from core resolution.

## Verification

- No new observability surfaces are added, but Twin Core state logging will now correctly trace back to blueprint emission via `custom_signals`.

## Tasks

- [x] **T01: Create Agumon and Gabumon blueprints** `est:30m`
  Create the Agumon and Gabumon blueprint modules to process their respective custom signals. These blueprints will translate signals like `apply_heated` or `apply_deep_crack` into generic `twin_core_added_tag_transition` emissions.
  - Files: `src/combat/blueprints/agumon.rs`, `src/combat/blueprints/gabumon.rs`, `src/combat/blueprints/mod.rs`, `src/combat/twin_core.rs`
  - Verify: cargo check

- [x] **T02: Migrate Twin Core skills to custom signals and verify** `est:30m`
  Update `skills.ron` to attach Twin Core custom signals to Agumon and Gabumon's skills, and assert in tests that action resolution correctly generates Twin Core tags.
  - Files: `assets/data/skills.ron`, `tests/twin_core_integration.rs`
  - Verify: cargo test --test twin_core_integration

## Files Likely Touched

- src/combat/blueprints/agumon.rs
- src/combat/blueprints/gabumon.rs
- src/combat/blueprints/mod.rs
- src/combat/twin_core.rs
- assets/data/skills.ron
- tests/twin_core_integration.rs
