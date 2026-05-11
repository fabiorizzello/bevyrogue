---
id: T01
parent: S08
milestone: M012
key_files:
  - src/data/units_ron.rs — added typed counterplay declarations, serde defaults, and parsing tests
  - assets/data/units.ron — annotated Devimon and Ogremon with explicit enemy counterplay declarations
  - tests/roster_catalog.rs — asserted canonical counterplay declarations and the Armored/ReactiveArmor guard
  - src/combat/bootstrap.rs — updated `taichi_def()` for the new `UnitDef` fields
  - tests/bootstrap_spawn_composition.rs — added default counterplay fields to inline roster fixtures
  - tests/tempo_resistance.rs — added default counterplay fields to boss/ally fixtures
  - tests/roster_smoke.rs — added default counterplay fields to inline enemies
  - tests/follow_up_chains.rs — added default counterplay fields to inline follow-up fixture
key_decisions:
  - Used explicit `EnemyCounterplayStatus::Deferred { reason }` values instead of inferring implementation from toughness metadata.
  - Kept `toughness_category: Armored` on Devimon to avoid conflating a defensive archetype with the new ReactiveArmor declaration.
  - Made the new declaration fields optional/defaulted at the serde boundary so older roster fixtures remain compatible.
duration: 
verification_result: passed
completed_at: 2026-05-01T14:27:19.746Z
blocker_discovered: false
---

# T01: Added explicit enemy counterplay declarations to unit data with typed status fields and canonical roster coverage.

**Added explicit enemy counterplay declarations to unit data with typed status fields and canonical roster coverage.**

## What Happened

Implemented typed enemy counterplay metadata in `src/data/units_ron.rs` by adding `EnemyCounterplayKind`, `EnemyCounterplayStatus`, `EnemyTraitDeclaration`, and `ChargedAttackDeclaration`, with `#[serde(default)]` on `UnitDef.enemy_traits` and `UnitDef.charged_attack` so older RON records still deserialize cleanly.

Updated the canonical roster in `assets/data/units.ron` so Devimon now declares TempoAnchor as implemented, TypeTrap and ReactiveArmor as deferred, and a deferred charged-attack telegraph; Ogremon carries a deferred charged-attack declaration; Goblimon remains empty. I also updated `taichi_def()` and every inline `UnitDef` fixture that needed the new fields to keep the crate compiling, and strengthened `tests/roster_catalog.rs` to assert the new declarations and the Armored-toughness guard explicitly.

Added a round-trip test for the new typed declarations plus a backward-compatibility parse test that omits the new fields entirely. Captured the main gotcha as durable project knowledge: `ReactiveArmor` must remain an explicit declaration and must not be inferred from `ToughnessCategory::Armored`.

## Verification

Ran `cargo test-dev --test roster_catalog && cargo test-dev units_ron` after the final code change. The roster catalog suite passed 2/2 tests, and the `units_ron` test target passed 7/7 tests; the combined command exited successfully.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test roster_catalog && cargo test-dev units_ron` | 0 | ✅ pass | 4500ms |

## Deviations

Expanded fixture literals in `tests/bootstrap_spawn_composition.rs`, `tests/tempo_resistance.rs`, `tests/roster_smoke.rs`, and `tests/follow_up_chains.rs` so the new `UnitDef` fields compile cleanly.

## Known Issues

None.

## Files Created/Modified

- `src/data/units_ron.rs — added typed counterplay declarations, serde defaults, and parsing tests`
- `assets/data/units.ron — annotated Devimon and Ogremon with explicit enemy counterplay declarations`
- `tests/roster_catalog.rs — asserted canonical counterplay declarations and the Armored/ReactiveArmor guard`
- `src/combat/bootstrap.rs — updated `taichi_def()` for the new `UnitDef` fields`
- `tests/bootstrap_spawn_composition.rs — added default counterplay fields to inline roster fixtures`
- `tests/tempo_resistance.rs — added default counterplay fields to boss/ally fixtures`
- `tests/roster_smoke.rs — added default counterplay fields to inline enemies`
- `tests/follow_up_chains.rs — added default counterplay fields to inline follow-up fixture`
