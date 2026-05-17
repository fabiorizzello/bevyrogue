# S10: Patamon + Renamon migrated + kernel digimon-free

**Goal:** Migrate Patamon and Renamon onto the shared Blueprint owner envelope, then remove remaining digimon-named runtime/event/observability seams from src/combat outside blueprints so the kernel is structurally digimon-free while headless and windowed proofs stay green.
**Demo:** Kernel digimon-free verificato grep; smoke UI 2 encounter.

## Must-Haves

- ## Must-Haves
- Patamon custom signals dispatch only `CombatKernelTransition::Blueprint { owner: "patamon", ... }`, with Holy Support state/runtime owned by `src/combat/blueprints/patamon/**`.
- Renamon custom signals dispatch only `CombatKernelTransition::Blueprint { owner: "renamon", ... }`, with Kitsune Grace / precision runtime owned by `src/combat/blueprints/renamon.rs` and no Renamon-specific runtime registration left in `src/combat/kernel.rs`.
- Shared combat files outside `src/combat/blueprints/**` no longer contain digimon mechanic names covered by the roadmap grep gate.
- Shared validation / CLI observability surfaces expose blueprint diagnostics generically rather than through digimon-named fields or event variants.
- ## Threat Surface
- **Abuse**: malformed or foreign Blueprint envelopes must stay no-op/rejected rather than mutating the wrong owner state.
- **Data exposure**: none; this slice changes local combat runtime/state diagnostics only.
- **Input trust**: untrusted inputs are RON-defined custom signals and runtime Blueprint envelopes flowing through the combat event bus.
- ## Requirement Impact
- **Requirements touched**: none preloaded.
- **Re-verify**: Patamon/Renamon signal dispatch, Tentomon/Dorumon regression paths, shared snapshot/CLI output, headless/windowed registration.
- **Decisions revisited**: D003.
- ## Proof Level
- This slice proves: integration
- Real runtime required: yes
- Human/UAT required: no
- ## Verification
- `rg 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat --glob '!blueprints/**'`
- `cargo test --test patamon_blueprint_seam`
- `cargo test --test holy_support_resolution`
- `cargo test --test digimon_signal_registry`
- `cargo test --test compiled_timeline_tohakken`
- `cargo test --test renamon_precision_runtime`
- `cargo test --test battery_loop_kernel`
- `cargo test --test dorumon_predator_runtime`
- `cargo test --test event_stream`
- `cargo test --test validation_snapshot`
- `cargo test --test combat_cli_shared_surface`
- `cargo check`
- `cargo check --features windowed`
- ## Observability / Diagnostics
- Runtime signals: shared `OnKernelTransition` stays generic; blueprint-owned runtime state/snapshots become the mechanic-specific diagnostic seam.
- Inspection surfaces: `capture_validation_snapshot`, formatted CLI proof output, targeted integration tests reading blueprint resources.
- Failure visibility: malformed/foreign owner envelopes remain observable through rejected or unchanged blueprint state rather than hidden kernel mutation.
- Redaction constraints: none.
- ## Integration Closure
- Upstream surfaces consumed: `src/combat/kernel.rs`, `src/combat/events.rs`, blueprint dispatch modules, validation snapshot formatting, `src/bin/combat_cli.rs`.
- New wiring introduced in this slice: Patamon/Renamon/Tentomon/Dorumon owner modules become the sole homes for their runtime transition decoding and diagnostics.
- What remains before the milestone is truly usable end-to-end: S11 still needs UI/AI preview consumers; S12 still needs roster/validation snapshot registry-keyed cleanup.

## Proof Level

- This slice proves: integration

## Integration Closure

After S10, combat shared runtime surfaces are digimon-free and all remaining per-owner mechanics live behind blueprint modules. Milestone closure still depends on S11 consuming SkillCtx preview in UI/AI paths and S12 keying roster/validation registration off blueprint ownership rather than hardcoded digimon metadata.

## Verification

- This slice changes the diagnostic contract from digimon-named shared fields/events to generic shared surfaces plus owner-owned blueprint state. Future agents should inspect blueprint resources and generic validation output instead of assuming `holy_support`, `battery_loop`, `predator_loop`, or `precision_mind_game` fields exist in shared combat modules.

## Tasks

- [x] **T01: Migrate Patamon Holy Support transport onto the Blueprint owner envelope** `est:1.5h`
  Skills used: bevy, rust-best-practices, verify-before-complete.
  - Files: `src/combat/blueprints/patamon/signals.rs`, `src/combat/blueprints/patamon/identity.rs`, `src/combat/blueprints/patamon/mod.rs`, `tests/patamon_blueprint_seam.rs`, `tests/holy_support_resolution.rs`
  - Verify: cargo test --test patamon_blueprint_seam
cargo test --test holy_support_resolution

- [ ] **T02: Move Renamon precision runtime ownership behind the blueprint envelope** `est:2h`
  Skills used: bevy, rust-best-practices, verify-before-complete.
  - Files: `src/combat/blueprints/renamon.rs`, `src/combat/kernel.rs`, `tests/digimon_signal_registry.rs`, `tests/compiled_timeline_tohakken.rs`, `tests/renamon_precision_runtime.rs`
  - Verify: cargo test --test digimon_signal_registry
cargo test --test compiled_timeline_tohakken
cargo test --test renamon_precision_runtime

- [ ] **T03: Remove digimon-named runtime and event seams from shared combat modules** `est:3h`
  Skills used: bevy, rust-best-practices, verify-before-complete.
  - Files: `src/combat/kernel.rs`, `src/combat/events.rs`, `src/combat/mod.rs`, `src/combat/api/applier.rs`, `src/combat/blueprints/tentomon.rs`, `src/combat/blueprints/dorumon/identity.rs`, `tests/battery_loop_kernel.rs`, `tests/dorumon_predator_runtime.rs`, `tests/event_stream.rs`
  - Verify: cargo test --test battery_loop_kernel
cargo test --test dorumon_predator_runtime
cargo test --test event_stream

- [ ] **T04: Genericize validation and CLI observability, then prove the kernel-free grep gate** `est:2.5h`
  Skills used: bevy, rust-best-practices, verify-before-complete.
  - Files: `src/combat/observability.rs`, `src/bin/combat_cli.rs`, `tests/validation_snapshot.rs`, `tests/combat_cli_shared_surface.rs`, `tests/patamon_blueprint_seam.rs`, `tests/holy_support_resolution.rs`, `tests/renamon_precision_runtime.rs`
  - Verify: rg 'TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace' src/combat --glob '!blueprints/**'
cargo test --test validation_snapshot
cargo test --test combat_cli_shared_surface
cargo check
cargo check --features windowed

## Files Likely Touched

- src/combat/blueprints/patamon/signals.rs
- src/combat/blueprints/patamon/identity.rs
- src/combat/blueprints/patamon/mod.rs
- tests/patamon_blueprint_seam.rs
- tests/holy_support_resolution.rs
- src/combat/blueprints/renamon.rs
- src/combat/kernel.rs
- tests/digimon_signal_registry.rs
- tests/compiled_timeline_tohakken.rs
- tests/renamon_precision_runtime.rs
- src/combat/events.rs
- src/combat/mod.rs
- src/combat/api/applier.rs
- src/combat/blueprints/tentomon.rs
- src/combat/blueprints/dorumon/identity.rs
- tests/battery_loop_kernel.rs
- tests/dorumon_predator_runtime.rs
- tests/event_stream.rs
- src/combat/observability.rs
- src/bin/combat_cli.rs
- tests/validation_snapshot.rs
- tests/combat_cli_shared_surface.rs
