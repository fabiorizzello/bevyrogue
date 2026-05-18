# S09: Dorumon + Tentomon migrated (Predator Loop + Battery Loop)

**Goal:** Migrate Dorumon Predator Loop and Tentomon Battery Loop off digimon-specific CombatKernelTransition variants onto the shared Blueprint owner path while preserving typed resolved events, deterministic passive behavior, and event-stream observability.
**Demo:** Predator Loop write in JSONL; Battery Loop deterministico.

## Must-Haves

- Raw Dorumon and Tentomon kernel writes flow through `CombatKernelTransition::Blueprint { owner, name, payload }` instead of `CombatKernelTransition::PredatorLoop` / `CombatKernelTransition::BatteryLoop`.
- `PredatorLoopResolved` and `BatteryLoopResolved` remain the typed post-application observability seam for state snapshots and downstream tests.
- Dorumon runtime coverage proves Predator Loop still applies exploit/prey-lock state after decoding generic Blueprint transitions.
- Tentomon runtime coverage proves Battery Loop state changes and block-reaction determinism are unchanged after the envelope rewrite.
- `cargo test --test dorumon_blueprint`
- `cargo test --test dorumon_predator_runtime`
- `cargo test --test tentomon_blueprint`
- `cargo test --test battery_loop_kernel`
- `cargo test --test passive_reactive_canon`
- `cargo test --test event_stream`
- `cargo check`
- `cargo check --features windowed`

## Proof Level

- This slice proves: Integration proof. Real runtime required: yes. Human/UAT required: no. The slice is done only when both blueprint dispatch paths produce generic Blueprint kernel events and the existing typed resolved-event/state seams stay green under executable tests.

## Integration Closure

Upstream surfaces consumed: `CombatKernelTransition::Blueprint`, Dorumon/Tentomon blueprint dispatch hooks, kernel runtime registration, combat event serialization, and validation snapshot formatting. New wiring introduced in this slice: Dorumon and Tentomon decode their own `owner/name/payload` Blueprint transitions at the blueprint-owned runtime boundary instead of relying on kernel-local transition variants. What remains before the milestone is truly usable end-to-end: S10 still needs Patamon/Renamon migration and the final kernel digimon-free grep closure.

## Verification

- Preserve `PredatorLoopResolved`, `BatteryLoopResolved`, validation snapshots, and passive block-reaction diagnostics while moving the raw transition write path to the generic Blueprint owner envelope. Event-stream assertions in this slice should pin both the raw Blueprint transition shape and the unchanged typed resolved-event surfaces so future regressions are localizable.

## Tasks

- [x] **T01: Move Dorumon Predator Loop onto Blueprint owner transitions** `est:1h`
  Skills: `bevy`, `rust-best-practices`, `tdd`.
  - Files: `src/combat/blueprints/dorumon/signals.rs`, `src/combat/blueprints/dorumon/hooks.rs`, `src/combat/blueprints/dorumon/identity.rs`, `tests/dorumon_blueprint.rs`, `tests/dorumon_predator_runtime.rs`
  - Verify: cargo test --test dorumon_blueprint
cargo test --test dorumon_predator_runtime

- [x] **T02: Move Tentomon Battery Loop onto Blueprint owner transitions** `est:1h`
  Skills: `bevy`, `rust-best-practices`, `tdd`.
  - Files: `src/combat/blueprints/tentomon.rs`, `src/combat/battery_loop.rs`, `tests/tentomon_blueprint.rs`, `tests/battery_loop_kernel.rs`, `tests/passive_reactive_canon.rs`
  - Verify: cargo test --test tentomon_blueprint
cargo test --test battery_loop_kernel
cargo test --test passive_reactive_canon

- [x] **T03: Remove kernel-local Predator/Battery transition ownership and update shared observability surfaces** `est:1h`
  Skills: `bevy`, `rust-best-practices`, `tdd`.
  - Files: `src/combat/kernel.rs`, `src/combat/events.rs`, `src/combat/observability.rs`, `src/combat/blueprints/dorumon/mod.rs`, `src/combat/blueprints/dorumon/identity.rs`, `src/combat/battery_loop.rs`, `tests/predator_loop_kernel.rs`, `tests/event_stream.rs`
  - Verify: cargo test --test event_stream
cargo test --test predator_loop_kernel
cargo check

- [x] **T04: Run the slice verification sweep across headless and windowed builds** `est:20m`
  Skills: `bevy`, `tdd`, `verify-before-complete`.
  - Verify: cargo test --test dorumon_blueprint
cargo test --test dorumon_predator_runtime
cargo test --test tentomon_blueprint
cargo test --test battery_loop_kernel
cargo test --test passive_reactive_canon
cargo test --test event_stream
cargo check
cargo check --features windowed

## Files Likely Touched

- src/combat/blueprints/dorumon/signals.rs
- src/combat/blueprints/dorumon/hooks.rs
- src/combat/blueprints/dorumon/identity.rs
- tests/dorumon_blueprint.rs
- tests/dorumon_predator_runtime.rs
- src/combat/blueprints/tentomon.rs
- src/combat/battery_loop.rs
- tests/tentomon_blueprint.rs
- tests/battery_loop_kernel.rs
- tests/passive_reactive_canon.rs
- src/combat/kernel.rs
- src/combat/events.rs
- src/combat/observability.rs
- src/combat/blueprints/dorumon/mod.rs
- tests/predator_loop_kernel.rs
- tests/event_stream.rs
