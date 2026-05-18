---
estimated_steps: 13
estimated_files: 8
skills_used: []
---

# T03: Remove kernel-local Predator/Battery transition ownership and update shared observability surfaces

Skills: `bevy`, `rust-best-practices`, `tdd`.

Why: After both blueprints speak the generic owner envelope, the remaining cleanup is to stop `kernel.rs` from owning Dorumon/Tentomon-specific transition types and to re-home imports so shared event/observability code reflects the new ownership boundary.

Do:
1. Move `BatteryLoop*` and `PredatorLoop*` transition/signal types out of `src/combat/kernel.rs` into their owning runtime modules, then update module exports so callers can still import the typed resolved-event payloads from stable combat namespaces.
2. Update `src/combat/events.rs` and `src/combat/observability.rs` to import the typed Battery Loop / Predator Loop transition types from their new owners instead of `kernel.rs`.
3. Refresh any affected runtime/unit tests, including event-stream assertions, so they cover the new import locations and prove the raw `OnKernelTransition` records now contain Blueprint owners for Twin Core, Dorumon, and Tentomon.
4. Keep headless-first runtime wiring unchanged; do not introduce windowed-only coupling while moving ownership.

Done when: `kernel.rs` no longer owns Dorumon/Tentomon-specific transition definitions or variants, shared events/observability compile against blueprint-owned types, and event-stream coverage asserts the generic Blueprint raw writes.

Failure modes / negative checks:
- Re-export churn must not break existing test imports or snapshot formatting.
- Event serialization must still include typed resolved events after the ownership move.
- Cross-blueprint owner dispatch must remain isolated so one applier cannot consume another owner’s transition.

Observability impact: Clarifies that kernel observability owns only generic transitions while blueprint-specific typed resolved events stay available through their feature owners.

## Inputs

- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/observability.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/battery_loop.rs`
- `tests/predator_loop_kernel.rs`
- `tests/event_stream.rs`

## Expected Output

- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/observability.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/battery_loop.rs`
- `tests/predator_loop_kernel.rs`
- `tests/event_stream.rs`

## Verification

cargo test --test event_stream
cargo test --test predator_loop_kernel
cargo check

## Observability Impact

Makes the ownership boundary explicit in the code while preserving event-stream and snapshot surfaces.
