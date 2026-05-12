# S01: Tentomon/Kabuterimon Battery Loop Blueprint

**Goal:** Migrate Tentomon/Kabuterimon Battery Loop to blueprint.
**Demo:** After this: Battery mechanics operate entirely through `custom_signals` and the Tentomon blueprint, with CLI proof passing.

## Must-Haves

- Battery loop logic extracted to Tentomon blueprint.

## Proof Level

- This slice proves: integration (ECS + CLI)

## Verification

- ValidationSnapshot captures BatteryLoopState.

## Tasks

- [x] **T01: Define Tentomon signals and update skill data** `est:30m`
  Add `TentomonCustomSignal` to the skill DSL and update `assets/data/skills.ron` for the Tentomon/Kabuterimon roster.
  - Files: `src/data/skills_ron.rs`, `assets/data/skills.ron`
  - Verify: cargo check

- [x] **T02: Implement Tentomon blueprint** `est:30m`
  Create the Tentomon blueprint to interpret custom signals.
  - Files: `src/combat/blueprints/tentomon.rs`, `src/combat/blueprints/mod.rs`
  - Verify: cargo check

- [x] **T03: Verify Tentomon blueprint** `est:30m`
  Verify the Tentomon blueprint via integration test.
  - Files: `tests/tentomon_blueprint.rs`, `src/combat/observability.rs`
  - Verify: cargo test --test tentomon_blueprint

## Files Likely Touched

- src/data/skills_ron.rs
- assets/data/skills.ron
- src/combat/blueprints/tentomon.rs
- src/combat/blueprints/mod.rs
- tests/tentomon_blueprint.rs
- src/combat/observability.rs
