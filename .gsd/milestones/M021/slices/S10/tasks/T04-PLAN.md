---
estimated_steps: 4
estimated_files: 7
skills_used: []
---

# T04: Genericize validation and CLI observability, then prove the kernel-free grep gate

Skills used: bevy, rust-best-practices, verify-before-complete.

Why: `src/combat/observability.rs` and the CLI/snapshot tests are the largest remaining blockers for the digimon-free shared-surface claim, and they currently hard-code `twin_core`, `holy_support`, `predator_loop`, `battery_loop`, and `precision_mind_game` fields/text.

Do: redesign validation snapshot capture/formatting so shared observability code reports blueprint diagnostics generically while still exposing enough detail for tests and future debugging; update CLI proof output and snapshot-focused tests to assert the generic contract; revise Patamon/Renamon-facing snapshot assertions if needed to match the new generic formatting; then run the structural grep plus targeted snapshot/CLI/build checks to lock the slice exit proof.

Done when: shared observability/CLI code no longer contains the roadmap grep names outside blueprints, snapshot/CLI proofs pass on the generic contract, and both headless and windowed build checks are green.

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

## Observability Impact

Validation snapshots and CLI proof output become the generic, owner-agnostic inspection surface for migrated blueprint mechanics.
