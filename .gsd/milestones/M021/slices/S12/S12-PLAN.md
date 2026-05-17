# S12: RosterEntry blueprint-keyed + ValidationSnapshot from registry

**Goal:** Remove digimon-named roster and validation seams by moving `UnitDef` blueprint metadata to an owner-keyed generic payload and by sourcing blueprint validation fields from `ExtRegistries` instead of hardcoded shared snapshot structs.
**Demo:** Test 'add new digimon' modifica solo le 2 dir; suite verde.

## Must-Haves

- Structural grep is clean: `rg -n "TwinCoreRosterMetadata|HolySupportRosterMetadata|pub twin_core:|pub holy_support:" src/data src/combat tests` returns no matches.
- Shared observability no longer hardcodes blueprint-owned sub-snapshots: `rg -n "ValidationTwinCoreSnapshot|holy_support: Option<|predator_loop: Option<|battery_loop: Option<|precision_mind_game: Option<" src/combat tests` returns no matches outside blueprint-local snapshot types.
- `capture_validation_snapshot()` no longer fails on missing `TwinCoreState`, and registry-driven validation output remains deterministic by owner/key ordering.
- Focused regressions pass: `cargo test --test roster_smoke --test bootstrap_spawn_composition --test validation_snapshot --test combat_cli_shared_surface --test predator_loop_kernel --test holy_support_affordance --test holy_support_mechanics --test holy_support_resolution --test patamon_blueprint_seam --test twin_core_integration --test dorumon_predator_runtime --test renamon_precision_runtime`.
- Both build modes stay green: `cargo check` and `cargo check --features windowed`.

## Proof Level

- This slice proves: Integration proof. Real runtime required: yes, because registry registration, world-resource capture, and CLI/headless formatting must exercise the actual Bevy resources and blueprint-owned state. Human/UAT required: no.

## Integration Closure

This slice closes the shared-schema and shared-observability part of M021 context items C2/C3: blueprints own their roster metadata decoding and validation emission, while shared consumers only iterate generic registry fields. Central blueprint registration and the monolithic `data/units.ron` loader remain separate composition seams; this plan does not expand S12 into auto-discovery or per-digimon asset-directory loading.

## Verification

- Validation capture becomes owner-keyed and registry-driven, so future failures surface as missing/empty owner sections instead of missing shared fields. Deterministic owner/key sorting keeps CLI/headless snapshots diff-friendly, and missing blueprint state should degrade to absent sections rather than `TwinCoreState` hard-fail crashes.

## Tasks

- [x] **T01: Replace digimon-named UnitDef metadata with owner-keyed blueprint roster entries** `est:1.0d`
  Why: `src/data/units_ron.rs` still couples the shared roster schema to `twin_core` and `holy_support`, and many constructors/tests still rely on those fields. Skills: design-an-interface, tdd, bevy, rust-best-practices. Do: introduce a generic owner-keyed blueprint metadata shape in `UnitDef` with deterministic serialized ordering; remove the digimon-named metadata structs/fields; update bootstrap/manual `UnitDef` constructors and roster-boundary tests to use the generic payload or empty defaults; keep parsing backward-compatible where practical and preserve headless-first determinism. Negative coverage should include parsing units that omit blueprint payloads entirely and round-tripping blueprint entry order without `HashMap` instability. Done when: shared data schema no longer names Twin Core or Holy Support, constructor fallout is resolved, and roster-focused regressions pass.
  - Files: `src/data/units_ron.rs`, `src/combat/bootstrap.rs`, `assets/data/units.ron`, `tests/roster_smoke.rs`, `tests/bootstrap_spawn_composition.rs`, `tests/holy_support_roster_contract.rs`, `tests/presentation_metadata_boundary.rs`, `tests/combat_coherence.rs`, `tests/follow_up_chains.rs`, `tests/follow_up_triggers.rs`, `tests/resource_caps.rs`, `tests/tempo_resistance.rs`, `tests/twin_core_integration.rs`
  - Verify: cargo test --test roster_smoke --test bootstrap_spawn_composition --test holy_support_roster_contract --test presentation_metadata_boundary

- [ ] **T02: Add ValidationExt and refactor validation capture to registry-owned blueprint contributors** `est:1.0d`
  Why: `capture_validation_snapshot()` still hardcodes blueprint-owned state and treats `TwinCoreState` as mandatory, which violates M021 C3. Skills: design-an-interface, tdd, bevy, rust-best-practices. Do: add a `ValidationExt` axis to `ExtRegistries`; define a small generic validation-field contract collected from `World`; register contributors from the Twin Core, Patamon, Dorumon, Tentomon, and Renamon blueprint ownership seams; refactor `ValidationSnapshot` and `format_validation_snapshot()` to store/render registry-produced owner sections with deterministic owner/key sorting; keep malformed/foreign owner data as no-op or absent diagnostics instead of shared crashes. Failure modes to cover: missing optional blueprint resources should render `none`/absence, foreign owners must not mutate shared state, and deterministic output must not depend on registration order. Done when: hardcoded blueprint snapshot members are gone from shared observability, `TwinCoreState` is optional, and focused validation/runtime tests pass.
  - Files: `src/combat/api/registry.rs`, `src/combat/observability.rs`, `src/combat/blueprints/mod.rs`, `src/combat/blueprints/agumon/mod.rs`, `src/combat/blueprints/gabumon/mod.rs`, `src/combat/blueprints/twin_core/mod.rs`, `src/combat/blueprints/patamon/mod.rs`, `src/combat/blueprints/dorumon/mod.rs`, `src/combat/blueprints/renamon.rs`, `src/combat/blueprints/tentomon.rs`, `tests/validation_snapshot.rs`, `tests/predator_loop_kernel.rs`, `tests/patamon_blueprint_seam.rs`, `tests/twin_core_integration.rs`, `tests/dorumon_predator_runtime.rs`, `tests/renamon_precision_runtime.rs`
  - Verify: cargo test --test validation_snapshot --test predator_loop_kernel --test patamon_blueprint_seam --test twin_core_integration --test dorumon_predator_runtime --test renamon_precision_runtime

- [ ] **T03: Realign CLI and shared-surface proofs to the generic validation contract** `est:0.5d`
  Why: once roster and validation become generic, the CLI/headless/windowed consumers and shared-surface tests must prove the new contract instead of the retired named fields. Skills: tdd, bevy, rust-best-practices, verify-before-complete. Do: update CLI/headless/windowed validation formatting call sites only as needed for the new snapshot shape; rewrite shared-surface and affordance assertions to check owner-keyed output and the absence of digimon-named shared fields; keep proof focused on executable boundary checks rather than roadmap-wide auto-discovery claims. Include negative checks that snapshot rendering stays stable when optional blueprint sections are absent. Done when: CLI/shared-surface regressions assert the new contract, structural greps are part of the final proof, and both cargo check modes stay green.
  - Files: `src/bin/combat_cli.rs`, `src/headless.rs`, `src/windowed.rs`, `tests/combat_cli_shared_surface.rs`, `tests/holy_support_affordance.rs`, `tests/holy_support_mechanics.rs`, `tests/holy_support_resolution.rs`, `tests/presentation_metadata_boundary.rs`
  - Verify: cargo test --test combat_cli_shared_surface --test holy_support_affordance --test holy_support_mechanics --test holy_support_resolution

## Files Likely Touched

- src/data/units_ron.rs
- src/combat/bootstrap.rs
- assets/data/units.ron
- tests/roster_smoke.rs
- tests/bootstrap_spawn_composition.rs
- tests/holy_support_roster_contract.rs
- tests/presentation_metadata_boundary.rs
- tests/combat_coherence.rs
- tests/follow_up_chains.rs
- tests/follow_up_triggers.rs
- tests/resource_caps.rs
- tests/tempo_resistance.rs
- tests/twin_core_integration.rs
- src/combat/api/registry.rs
- src/combat/observability.rs
- src/combat/blueprints/mod.rs
- src/combat/blueprints/agumon/mod.rs
- src/combat/blueprints/gabumon/mod.rs
- src/combat/blueprints/twin_core/mod.rs
- src/combat/blueprints/patamon/mod.rs
- src/combat/blueprints/dorumon/mod.rs
- src/combat/blueprints/renamon.rs
- src/combat/blueprints/tentomon.rs
- tests/validation_snapshot.rs
- tests/predator_loop_kernel.rs
- tests/patamon_blueprint_seam.rs
- tests/dorumon_predator_runtime.rs
- tests/renamon_precision_runtime.rs
- src/bin/combat_cli.rs
- src/headless.rs
- src/windowed.rs
- tests/combat_cli_shared_surface.rs
- tests/holy_support_affordance.rs
- tests/holy_support_mechanics.rs
- tests/holy_support_resolution.rs
