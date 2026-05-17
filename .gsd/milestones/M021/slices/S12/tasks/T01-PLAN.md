---
estimated_steps: 1
estimated_files: 13
skills_used: []
---

# T01: Replace digimon-named UnitDef metadata with owner-keyed blueprint roster entries

Why: `src/data/units_ron.rs` still couples the shared roster schema to `twin_core` and `holy_support`, and many constructors/tests still rely on those fields. Skills: design-an-interface, tdd, bevy, rust-best-practices. Do: introduce a generic owner-keyed blueprint metadata shape in `UnitDef` with deterministic serialized ordering; remove the digimon-named metadata structs/fields; update bootstrap/manual `UnitDef` constructors and roster-boundary tests to use the generic payload or empty defaults; keep parsing backward-compatible where practical and preserve headless-first determinism. Negative coverage should include parsing units that omit blueprint payloads entirely and round-tripping blueprint entry order without `HashMap` instability. Done when: shared data schema no longer names Twin Core or Holy Support, constructor fallout is resolved, and roster-focused regressions pass.

## Inputs

- `src/data/units_ron.rs`
- `src/combat/bootstrap.rs`
- `assets/data/units.ron`
- `tests/roster_smoke.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/holy_support_roster_contract.rs`
- `tests/presentation_metadata_boundary.rs`

## Expected Output

- `src/data/units_ron.rs`
- `src/combat/bootstrap.rs`
- `assets/data/units.ron`
- `tests/roster_smoke.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/holy_support_roster_contract.rs`
- `tests/presentation_metadata_boundary.rs`
- `tests/combat_coherence.rs`
- `tests/follow_up_chains.rs`
- `tests/follow_up_triggers.rs`
- `tests/resource_caps.rs`
- `tests/tempo_resistance.rs`
- `tests/twin_core_integration.rs`

## Verification

cargo test --test roster_smoke --test bootstrap_spawn_composition --test holy_support_roster_contract --test presentation_metadata_boundary

## Observability Impact

Blueprint roster payload parsing becomes the durable contract future agents inspect in schema and contract tests, reducing hidden coupling to shared digimon-named defaults.
