---
estimated_steps: 5
estimated_files: 4
skills_used:
  - tdd
  - test
  - verify-before-complete
---

# T01: Add typed enemy counterplay declarations to unit data

Why: S08 must stop treating free-text `signature_traits` as a UI contract. This task creates the typed RON/schema surface that later tasks can propagate into runtime snapshots. Keep `signature_traits` intact for flavor/catalog checks, but make `enemy_traits` and `charged_attack` the only machine-readable enemy counterplay declarations.

Failure Modes (Q5): Existing RON fixtures and helper constructors can fail to deserialize/compile if new fields lack `#[serde(default)]`; avoid that by defaulting optional/vector declaration fields. Canonical data can falsely imply implementation if `ReactiveArmor` is mapped from `ToughnessCategory::Armored`; explicitly avoid that mapping and encode it as deferred if declared.

Load Profile (Q6): Static roster parsing is in-memory and small; per-operation cost is trivial clone/deserialize work. At 10x roster size, readability and validation clarity fail before runtime performance.

Negative Tests (Q7): Cover backward-compatible parsing when declaration fields are omitted, canonical enemy declarations with expected implemented/deferred statuses, and a guard that Armored toughness is not interpreted as implemented Reactive Armor.

Implement typed declarations in or near `src/data/units_ron.rs`: `EnemyCounterplayKind` (`TypeTrap`, `ReactiveArmor`, `BreakSeal`, `TempoAnchor`), a reusable declaration status mirroring implemented/deferred/hidden with `LegalityReasonCode`, `EnemyTraitDeclaration`, and `ChargedAttackDeclaration` with `SkillId`, `lead_turns`, and status. Add `#[serde(default)] pub enemy_traits: Vec<EnemyTraitDeclaration>` and `#[serde(default)] pub charged_attack: Option<ChargedAttackDeclaration>` to `UnitDef`. Update `round_trip_unit_def()`, `taichi_def()`/test fixtures that construct `UnitDef`, and canonical `assets/data/units.ron` so Devimon declares `TempoAnchor` implemented plus deferred Type Trap/Reactive Armor/charged attack if present, while Ogremon can carry a deferred charged attack declaration and Goblimon remains empty. Do not change canonical `toughness_category` to `Shielded` just to prove Break Seal; that proof belongs in fixture query tests to avoid boss TTK regressions.

## Inputs

- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- `tests/roster_catalog.rs`

## Expected Output

- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `tests/roster_catalog.rs`

## Verification

cargo test-dev --test roster_catalog && cargo test-dev units_ron
