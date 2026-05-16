# S07: Modifier pipeline + Migrate 6 passive canon

**Goal:** Introduce the generic passive modifier pipeline, extend PassiveRunner routing beyond ult-only blueprint signals, and migrate the 6 canonical passives so Tentomon's Block Reaction resolves deterministically before the standard damage-reduction cascade.
**Demo:** 6 passive via PassiveRunner; Block Reaction verde deterministico.

## Must-Haves

- # S07: Modifier pipeline + Migrate 6 passive canon
- **Goal:** Introduce the shared passive modifier pipeline and migrate the six canonical passives onto PassiveRunner-backed runtime wiring.
- **Demo:** Agumon, Gabumon, Dorumon, Tentomon, Patamon, and Renamon all install through the runtime; Tentomon's Block Reaction triggers deterministically before the normal DR cascade; existing kernel identity resources remain the single state owners.
- ## Must-Haves
- All six canon passives run through shared PassiveRunner/filter wiring rather than bespoke one-off test scaffolds.
- IncomingDamage and BlockReactionTriggered surface the pre-damage mitigation seam needed for deterministic Tentomon behavior.
- ## Threat Surface
- **Abuse**: Internal event/filter misrouting or recursive passive cascades could cause duplicate reactions, infinite loops, or misordered mitigation.
- **Data exposure**: none.
- **Input trust**: Only trusted in-process combat events, skill assets, and deterministic RNG inputs are consumed.
- ## Requirement Impact
- **Requirements touched**: none recorded yet for this milestone.
- **Re-verify**: passive runtime wiring, pre-damage mitigation ordering, JSONL-safe combat event serialization, and existing Twin Core/Holy Support/Predator/Battery snapshot surfaces.
- **Decisions revisited**: none.
- ## Proof Level
- This slice proves: integration.
- Real runtime required: yes.
- Human/UAT required: no.
- ## Verification
- `cargo test --test passive_event_filters`
- `cargo test --test block_reaction_pipeline`
- `cargo test --test passive_canon_support`
- `cargo test --test passive_reactive_canon`
- `cargo test`
- `cargo check --features windowed`
- ## Observability / Diagnostics
- Runtime signals: `IncomingDamage`, `BlockReactionTriggered`, existing `OnKernelTransition::Blueprint`, and PassiveRunner circuit-breaker warnings.
- Inspection surfaces: combat event stream, JSONL serialization paths, validation snapshot formatters for holy_support/predator_loop/battery_loop, and targeted integration tests.
- Failure visibility: misordered mitigation shows up as wrong damage totals or missing reaction events; runaway listener cascades trip the existing hop breaker.
- Redaction constraints: none.
- ## Integration Closure
- Upstream surfaces consumed: `intent_applier`, `calculate_damage`, `CombatEvent`, `SignalBus`, `PassiveRunner`, and the Agumon/Patamon/Dorumon battery/predator identity resources.
- New wiring introduced in this slice: composite passive event routing, pre-damage modifier aggregation, and runtime-installed passive listeners for the six canon digimon.
- What remains before the milestone is truly usable end-to-end: S08-S10 still need roster-driven digimon migration cleanup and final kernel digimon-free verification.

## Proof Level

- This slice proves: This slice proves integration-level passive composition across the shared listener/filter layer, pre-damage modifier aggregation, and deterministic reactive mitigation in the real headless combat runtime; no human UAT is required.

## Integration Closure

Consumes the shared combat runtime (`CombatPlugin`, `intent_applier`, `SignalBus`, `PassiveRunner`, damage/events) and existing identity resources (Twin Core, Holy Support, Predator Loop, Battery Loop). Introduces runtime-installed passive listeners and the pre-damage mitigation seam, while leaving S08-S10 to finish roster migration and final kernel digimon-free cleanup.

## Verification

- Adds first-class passive-routing and pre-damage reaction signals (`IncomingDamage`, `BlockReactionTriggered`) while reusing existing Twin Core/Holy Support/Predator/Battery validation snapshot surfaces and PassiveRunner circuit-breaker warnings for diagnostics.

## Tasks

- [x] **T01: Add composite passive event routing and loop-capable PassiveRunner** `est:1h30m`
  Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/api/event_filter.rs`, `src/combat/api/event_bridge.rs`, `src/combat/api/passive_runner.rs`, `src/combat/api/signal.rs`, `src/combat/api/mod.rs`, `tests/passive_event_filters.rs`
  - Verify: cargo test --test passive_event_filters

- [ ] **T02: Wire modifier aggregation and pre-damage Block Reaction into damage application** `est:2h`
  Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/modifiers.rs`, `src/combat/api/intent.rs`, `src/combat/api/applier.rs`, `src/combat/damage.rs`, `src/combat/events.rs`, `src/combat/mod.rs`, `tests/block_reaction_pipeline.rs`
  - Verify: cargo test --test block_reaction_pipeline

- [ ] **T03: Migrate Agumon, Gabumon, Patamon, and Renamon passives onto PassiveRunner** `est:2h`
  Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/blueprints/agumon/mod.rs`, `src/combat/blueprints/gabumon.rs`, `src/combat/blueprints/patamon/mod.rs`, `src/combat/blueprints/renamon.rs`, `src/combat/kernel.rs`, `tests/passive_canon_support.rs`
  - Verify: cargo test --test passive_canon_support

- [ ] **T04: Migrate Dorumon and Tentomon reactive passives and prove deterministic Block Reaction** `est:2h`
  Expected skills: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/blueprints/dorumon/mod.rs`, `src/combat/blueprints/dorumon/hooks.rs`, `src/combat/blueprints/tentomon.rs`, `src/combat/battery_loop.rs`, `src/combat/kernel.rs`, `tests/passive_reactive_canon.rs`
  - Verify: cargo test --test passive_reactive_canon

## Files Likely Touched

- src/combat/api/event_filter.rs
- src/combat/api/event_bridge.rs
- src/combat/api/passive_runner.rs
- src/combat/api/signal.rs
- src/combat/api/mod.rs
- tests/passive_event_filters.rs
- src/combat/modifiers.rs
- src/combat/api/intent.rs
- src/combat/api/applier.rs
- src/combat/damage.rs
- src/combat/events.rs
- src/combat/mod.rs
- tests/block_reaction_pipeline.rs
- src/combat/blueprints/agumon/mod.rs
- src/combat/blueprints/gabumon.rs
- src/combat/blueprints/patamon/mod.rs
- src/combat/blueprints/renamon.rs
- src/combat/kernel.rs
- tests/passive_canon_support.rs
- src/combat/blueprints/dorumon/mod.rs
- src/combat/blueprints/dorumon/hooks.rs
- src/combat/blueprints/tentomon.rs
- src/combat/battery_loop.rs
- tests/passive_reactive_canon.rs
