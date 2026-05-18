---
estimated_steps: 1
estimated_files: 7
skills_used: []
---

# T05: Genericize validation and CLI observability, then prove the kernel-free grep gate

Once the Dorumon runtime contract is stable, finish the slice exit work by making validation snapshots and CLI proof output owner-agnostic, removing remaining digimon mechanic names from shared observability surfaces outside blueprints, and updating snapshot/CLI plus Patamon/Renamon-facing assertions to the generic diagnostic contract. Close the slice with the structural grep gate and both headless/windowed build checks.

## Inputs

- `src/combat/observability.rs`
- `src/bin/combat_cli.rs`
- `tests/validation_snapshot.rs`
- `tests/combat_cli_shared_surface.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/holy_support_resolution.rs`
- `tests/renamon_precision_runtime.rs`
- `src/combat/kernel.rs`
- `src/combat/events.rs`

## Expected Output

- `src/combat/observability.rs`
- `src/bin/combat_cli.rs`
- `tests/validation_snapshot.rs`
- `tests/combat_cli_shared_surface.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/holy_support_resolution.rs`
- `tests/renamon_precision_runtime.rs`

## Verification

rg 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat --glob '!blueprints/**'
cargo test --test validation_snapshot
cargo test --test combat_cli_shared_surface
cargo check
cargo check --features windowed
