# S08: Agumon + Gabumon migrated (Twin Core paired)

**Goal:** Agumon and Gabumon fully migrated onto kernel framework with Twin Core extracted to a shared mini-plugin module (no digimon coupling in kernel.rs for TwinCore), Gabumon restructured as directory module, and Bouncing Fire Loop branch gated behind a talent predicate proving OFF=baseline.
**Demo:** Twin Core end-to-end; Bouncing Fire OFF≡baseline; no coupling.

## Must-Haves

- 1. `rg "TwinCore" src/combat/ --glob '!blueprints/**'` → 0 lines (P001 for TwinCore)\n2. `cargo test twin_core` passes (integration + mechanics)\n3. `cargo test bouncing_fire` passes (OFF=baseline + rank1 hop)\n4. `cargo test` full suite green\n5. `cargo check --features windowed` clean\n6. Gabumon imports from `blueprints::twin_core::` not `blueprints::agumon::`

## Proof Level

- This slice proves: integration — tests exercise real timeline runner, PassiveRunner, and kernel transition dispatch through the Blueprint event path

## Integration Closure

Upstream: S06 (CompiledTimeline + ExtRegistries active skills), S07 (PassiveRunner + EventFilter passive wiring). New wiring: TwinCorePlugin replaces AgumonPlugin for shared state; Agumon/Gabumon blueprints both route through `CombatKernelTransition::Blueprint { owner: twin_core }`. Remaining: S09 (Dorumon+Tentomon), S10 (Patamon+Renamon + kernel digimon-free), S11 (UI/AI), S12 (RosterEntry).

## Verification

- TwinCore transitions now flow through `CombatKernelTransition::Blueprint { owner: "twin_core", ... }` in JSONL event stream — same observability surface as S07 passives, unified diagnostic path.

## Tasks

- [x] **T01: Extract TwinCore to blueprints/twin_core mini-plugin and remove kernel coupling** `est:2h`
  **Why:** The M021 success criterion requires `rg TwinCore src/combat/ --glob '!blueprints/**'` → 0 lines. Currently `TwinCoreSignal`, `TwinCoreTransition`, and the `CombatKernelTransition::TwinCore(...)` variant live in `kernel.rs`, and `TwinCoreHook`, `TwinCoreState`, `apply_twin_core_transitions_system` live in `blueprints/agumon/identity.rs`. All must consolidate into a new `blueprints/twin_core/` module. The `CombatKernelTransition::TwinCore(TwinCoreTransition)` variant is replaced by `CombatKernelTransition::Blueprint { owner: "twin_core", name: "<signal>", payload: Amount(amount) }` per M021 CONTEXT M5.
  - Files: `src/combat/blueprints/twin_core/mod.rs`, `src/combat/blueprints/mod.rs`, `src/combat/blueprints/agumon/identity.rs`, `src/combat/blueprints/agumon/mod.rs`, `src/combat/kernel.rs`, `src/combat/observability.rs`, `tests/twin_core_integration.rs`, `tests/twin_core_mechanics.rs`, `tests/event_stream.rs`, `tests/validation_snapshot.rs`, `tests/status_observability_canon.rs`, `tests/holy_support_mechanics.rs`, `tests/holy_support_affordance.rs`
  - Verify: cargo test && rg "TwinCore" src/combat/ --glob '!blueprints/**'

- [x] **T02: Convert Gabumon to directory module with twin_core imports** `est:30m`
  **Why:** Gabumon currently lives in a single flat file (`gabumon.rs`) and imports TwinCore types from `blueprints::agumon::`. After T01, those types live in `blueprints::twin_core::`. This task restructures Gabumon into a proper directory module (matching Agumon's structure) and fixes the import coupling.
  - Files: `src/combat/blueprints/gabumon/mod.rs`, `src/combat/blueprints/gabumon/signals.rs`
  - Verify: cargo test && rg "blueprints::agumon" src/combat/blueprints/gabumon/

- [x] **T03: Add Bouncing Fire Loop branch to baby_flame and register predicate+selector+hook** `est:1h30m`
  **Why:** The S08 success criterion requires 'Bouncing Fire OFF≡baseline' — proving that with talent rank 0 the intent stream from baby_flame is identical to the current no-loop timeline. This is the first production use of `BeatKind::Loop` in a real blueprint. It proves the Loop infrastructure from S02/S03 works in a real Digimon context and the gate mechanism cleanly gates off the branch.
  - Files: `src/combat/blueprints/agumon/mod.rs`, `assets/data/skills.ron`, `src/combat/api/registry.rs`
  - Verify: cargo check

- [ ] **T04: Write deterministic end-to-end tests for Twin Core Blueprint path and Bouncing Fire OFF=baseline** `est:1h`
  **Why:** The twin_core_integration and twin_core_mechanics tests currently pump `CombatKernelTransition::TwinCore(...)` directly. After T01 that variant is gone. T01 already updates imports, but this task ensures the tests exercise the new `Blueprint { owner: \"twin_core\" }` event path end-to-end, and adds the Bouncing Fire deterministic test.
  - Files: `tests/bouncing_fire_off_baseline.rs`, `tests/twin_core_integration.rs`, `tests/twin_core_mechanics.rs`
  - Verify: cargo test --test bouncing_fire_off_baseline && cargo test twin_core && cargo test

## Files Likely Touched

- src/combat/blueprints/twin_core/mod.rs
- src/combat/blueprints/mod.rs
- src/combat/blueprints/agumon/identity.rs
- src/combat/blueprints/agumon/mod.rs
- src/combat/kernel.rs
- src/combat/observability.rs
- tests/twin_core_integration.rs
- tests/twin_core_mechanics.rs
- tests/event_stream.rs
- tests/validation_snapshot.rs
- tests/status_observability_canon.rs
- tests/holy_support_mechanics.rs
- tests/holy_support_affordance.rs
- src/combat/blueprints/gabumon/mod.rs
- src/combat/blueprints/gabumon/signals.rs
- assets/data/skills.ron
- src/combat/api/registry.rs
- tests/bouncing_fire_off_baseline.rs
