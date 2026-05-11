---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Verify blueprint integration and CLI proof

Create `tests/tentomon_blueprint.rs` to verify that executing a skill with the `TentomonCustomSignal` correctly triggers the expected `BatteryLoopState` transitions through the kernel. Ensure no regressions occur in headless testing or the CLI proof.

## Inputs

- `src/combat/blueprints/tentomon.rs`

## Expected Output

- `tests/tentomon_blueprint.rs`

## Verification

cargo test --test tentomon_blueprint && BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli

## Observability Impact

Test coverage adds explicit guarantees around the blueprint-to-kernel seam.
