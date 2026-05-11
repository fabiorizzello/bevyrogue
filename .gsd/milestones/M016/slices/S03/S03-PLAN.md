# S03: S03

**Goal:** Migrate Renamon/Kyubimon precision loop logic into a dedicated blueprint, replacing ad-hoc kernel calls with owner-keyed custom signals.
**Demo:** After this: Precision loop logic operates through the Renamon blueprint.

## Must-Haves

- src/combat/blueprints/renamon.rs exists and is registered.
- assets/data/skills.ron uses custom_signals for Renamon/Kyubimon precision moves.
- tests/renamon_precision_runtime.rs passes and verifies state mutation.
- scripts/verify_combat_authority_audit.py passes.

## Proof Level

- This slice proves: Headless integration proof with validation snapshots.

## Integration Closure

The precision loop state machine in the kernel is now driven by the Renamon blueprint, which decodes custom signals from RON. This completes the migration of the Precision Mind-Game mechanic into the roster-specific blueprint layer.

## Verification

- Precision loop transitions are now visible via owner-keyed custom signals in action logs and reflected in the PrecisionMindGameState snapshot within the ValidationSnapshot.

## Tasks

- [x] **T01: Renamon blueprint skeleton & registration complete.** `est:30m`
  Establish the blueprint seam for the Renamon line and register it in the central registry. Add a basic test case to ensure the routing works.
  - Files: `src/combat/blueprints/renamon.rs`, `src/combat/blueprints/mod.rs`, `tests/digimon_signal_registry.rs`
  - Verify: cargo test --test digimon_signal_registry

- [x] **T02: Implement Renamon blueprint precision signal mapping logic.** `est:1h`
  Implement the translation from custom signals (e.g., open_momentum_window, commit_press) to PrecisionMindGameTransition kernel transitions within the Renamon blueprint.
  - Files: `src/combat/blueprints/renamon.rs`
  - Verify: cargo check

- [x] **T03: Update renamon/kyubimon skills with precision signals.** `est:45m`
  Update Renamon and Kyubimon skill definitions in assets/data/skills.ron to use the new custom_signals instead of any remaining ad-hoc logic.
  - Files: `assets/data/skills.ron`
  - Verify: cargo test --test digimon_signal_registry

- [x] **T04: Implement headless runtime proof for Renamon precision loop and restore missing blueprint registration.** `est:1h 30m`
  Implement a headless integration test that spawns Renamon/Kyubimon units, executes precision-based skills, and verifies that the PrecisionMindGameState advances correctly through the validation snapshot surface.
  - Files: `tests/renamon_precision_runtime.rs`
  - Verify: cargo test --test renamon_precision_runtime

## Files Likely Touched

- src/combat/blueprints/renamon.rs
- src/combat/blueprints/mod.rs
- tests/digimon_signal_registry.rs
- assets/data/skills.ron
- tests/renamon_precision_runtime.rs
