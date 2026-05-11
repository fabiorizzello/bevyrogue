---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Implement headless runtime proof for Renamon precision loop and restore missing blueprint registration.

Implement a headless integration test that spawns Renamon/Kyubimon units, executes precision-based skills, and verifies that the PrecisionMindGameState advances correctly through the validation snapshot surface.

## Inputs

- `src/combat/observability.rs`
- `tests/validation_snapshot.rs`

## Expected Output

- ``tests/renamon_precision_runtime.rs``

## Verification

cargo test --test renamon_precision_runtime
