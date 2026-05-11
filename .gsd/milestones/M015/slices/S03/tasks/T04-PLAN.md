---
estimated_steps: 14
estimated_files: 8
skills_used: []
---

# T04: Seed Patamon blueprint dispatch into Holy Support kernel state

Create the first concrete per-Digimon blueprint seam by letting Patamon Rust logic translate its RON signal into generic kernel transitions that the existing Holy Support machinery applies and exposes through events/snapshots.

Skills expected: `design-an-interface`, `tdd`, `verify-before-complete`.

Steps:
1. Add `src/combat/blueprints/mod.rs` as a tiny router/helper that delegates to per-Digimon modules and does not own Digimon-specific behavior itself.
2. Add `src/combat/blueprints/patamon.rs` to interpret the Patamon custom signal and return generic `CombatKernelTransition` values suitable for Holy Support hooks/appliers.
3. Export the new `blueprints` module from `src/combat/mod.rs` and call it from the shared action pipeline after a successful action, dispatching through `CombatKernelRegistry` and writing canonical `OnKernelTransition` events.
4. Expand Holy Support tests so they prove registry dispatch, event emission, applied `HolySupportState`, and `ValidationSnapshot` output from the Patamon action.

Must-haves:
- No Patamon or skill-ID-specific behavior is added to `resolution.rs`, `kernel.rs`, or generic action query code except for generic routing hooks.
- Holy Support shared primitives remain reusable; Patamon owns only the unique interpretation of its signal.
- Tests call `app.update()` where Bevy message/resource flushing is required before assertions.

Failure Modes (Q5): missing registry resources should produce a clear no-transition/no-state test failure; duplicate applier systems can double-apply Holy Support state; putting behavior in the router or kernel creates the drift S03 is meant to remove.
Load Profile (Q6): blueprint dispatch should be O(number of copied custom signals) per resolved action and should not scan all skills or all units.
Negative Tests (Q7): tests should cover a non-Patamon/no-signal action producing no Patamon transitions and the Patamon signal producing exactly the expected Holy Support transition/state.

## Inputs

- ``src/data/skills_ron.rs` — custom signal types added in T03.`
- ``src/combat/state.rs` — resolved-action custom signal payload added in T03.`
- ``src/combat/kernel.rs` — canonical transition/event contract and Holy Support hook/registry surface.`
- ``src/combat/holy_support.rs` — shared Holy Support state/applier primitive.`
- ``src/combat/turn_system/pipeline.rs` — shared runtime/test action pipeline from T02.`
- ``tests/patamon_blueprint_seam.rs` — parse/propagation proof to extend into full kernel/snapshot proof.`

## Expected Output

- ``src/combat/blueprints/mod.rs` — generic blueprint routing helper with no embedded Patamon behavior beyond delegation.`
- ``src/combat/blueprints/patamon.rs` — Patamon-specific signal interpretation into generic kernel transitions.`
- ``src/combat/mod.rs` — blueprint module exported for runtime/tests.`
- ``src/combat/turn_system/pipeline.rs` — successful actions dispatch blueprint transitions through the kernel registry.`
- ``tests/patamon_blueprint_seam.rs` — full end-to-end Patamon signal -> transition -> event -> state -> snapshot proof.`
- ``tests/holy_support_resolution.rs` — stale direct-effect expectations replaced with blueprint/kernel contract expectations.`
- ``tests/holy_support_roster_contract.rs` — stale removed-effect schema expectations replaced with custom-signal contract expectations.`

## Verification

cargo test --test patamon_blueprint_seam --test holy_support_resolution --test holy_support_roster_contract

## Observability Impact

Signals added/changed: blueprint-dispatched `OnKernelTransition` events for the Patamon/Holy Support seam. How a future agent inspects this: run `cargo test --test patamon_blueprint_seam` and inspect `format_validation_snapshot` assertions. Failure state exposed: missing custom signal, missing blueprint delegation, missing registry expansion, or missing Holy Support state is localized by separate assertions.
