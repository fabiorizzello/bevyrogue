---
estimated_steps: 1
estimated_files: 16
skills_used: []
---

# T02: Add ValidationExt and refactor validation capture to registry-owned blueprint contributors

Why: `capture_validation_snapshot()` still hardcodes blueprint-owned state and treats `TwinCoreState` as mandatory, which violates M021 C3. Skills: design-an-interface, tdd, bevy, rust-best-practices. Do: add a `ValidationExt` axis to `ExtRegistries`; define a small generic validation-field contract collected from `World`; register contributors from the Twin Core, Patamon, Dorumon, Tentomon, and Renamon blueprint ownership seams; refactor `ValidationSnapshot` and `format_validation_snapshot()` to store/render registry-produced owner sections with deterministic owner/key sorting; keep malformed/foreign owner data as no-op or absent diagnostics instead of shared crashes. Failure modes to cover: missing optional blueprint resources should render `none`/absence, foreign owners must not mutate shared state, and deterministic output must not depend on registration order. Done when: hardcoded blueprint snapshot members are gone from shared observability, `TwinCoreState` is optional, and focused validation/runtime tests pass.

## Inputs

- `src/combat/api/registry.rs`
- `src/combat/observability.rs`
- `src/combat/blueprints/mod.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/gabumon/mod.rs`
- `src/combat/blueprints/twin_core/mod.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/renamon.rs`
- `src/combat/blueprints/tentomon.rs`
- `tests/validation_snapshot.rs`
- `tests/predator_loop_kernel.rs`

## Expected Output

- `src/combat/api/registry.rs`
- `src/combat/observability.rs`
- `src/combat/blueprints/mod.rs`
- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/blueprints/gabumon/mod.rs`
- `src/combat/blueprints/twin_core/mod.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/renamon.rs`
- `src/combat/blueprints/tentomon.rs`
- `tests/validation_snapshot.rs`
- `tests/predator_loop_kernel.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/twin_core_integration.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/renamon_precision_runtime.rs`

## Verification

cargo test --test validation_snapshot --test predator_loop_kernel --test patamon_blueprint_seam --test twin_core_integration --test dorumon_predator_runtime --test renamon_precision_runtime

## Observability Impact

Adds a new inspection seam through `ExtRegistries` so blueprint diagnostics are discoverable by owner and degrade cleanly when state/resources are absent.
