---
id: T01
parent: S03
milestone: M011
key_files:
  - src/combat/types.rs
  - src/data/units_ron.rs
  - assets/data/units.ron
  - src/combat/bootstrap.rs
  - tests/bootstrap_spawn_composition.rs
  - tests/roster_smoke.rs
  - tests/follow_up_reentrancy.rs
key_decisions:
  - EvoLineId wraps String (no Copy) per spec — ensures it can't be accidentally reused as a trivial copy type when we later attach more data to lineage
  - taichi_def() uses sentinel EvoStage::Child + EvoLineId('tamer') — human commanders don't participate in evo chains; sentinel avoids Option<> complexity in the schema
  - Failing-parse test RON strings were extended with valid evo fields so each test remains scoped to its intended failure mode only
duration: 
verification_result: passed
completed_at: 2026-04-27T14:48:40.882Z
blocker_discovered: false
---

# T01: Added EvoStage 7-variant enum (JP naming) + EvoLineId newtype to types.rs, added 3 mandatory UnitDef fields, migrated all 12 units.ron entries and all Rust construction sites

**Added EvoStage 7-variant enum (JP naming) + EvoLineId newtype to types.rs, added 3 mandatory UnitDef fields, migrated all 12 units.ron entries and all Rust construction sites**

## What Happened

Added `EvoStage` (BabyI, BabyII, Child, Adult, Perfect, Ultimate, SuperUltimate) and `EvoLineId(pub String)` to `src/combat/types.rs`. EvoStage derives Copy; EvoLineId wraps String (no Copy per spec).

Added three mandatory fields to `UnitDef` in `src/data/units_ron.rs`: `evo_stage: EvoStage`, `evo_line: EvoLineId`, `evolves_to: Vec<UnitId>`. No serde default — parser fails loudly on any legacy RON missing these fields.

Updated all 12 entries in `assets/data/units.ron`: the 6 Child forms (Agumon, Gabumon, Dorumon, Renamon, Patamon, Tentomon) have `evo_stage: Child`, their respective `*_line` EvoLineId, and `evolves_to` pointing to their Adult UnitId. The 6 Adult forms (Greymon, Garurumon, Kabuterimon, Kyubimon, DORUgamon, Angemon) have `evo_stage: Adult`, same `*_line`, and `evolves_to: []`.

Updated `taichi_def()` in `bootstrap.rs` with sentinel values (`EvoStage::Child`, `EvoLineId("tamer")`, `evolves_to: vec![]`).

Updated all UnitDef construction sites in tests with `EvoStage::Child` / `EvoLineId("test")` / `evolves_to: vec![]`. Updated `round_trip_unit_def` inline literal with real values. Updated both failing-parse test RON strings to include the new mandatory evo fields (so they fail only on their intended bad input, not on missing evo_stage).

## Verification

Ran `cargo test --no-fail-fast` — all test binaries compiled and all tests passed (zero failures). Ran `grep -c 'evo_stage' assets/data/units.ron` — returned 12, confirming every unit entry has the field.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-fail-fast 2>&1 | grep 'test result'` | 0 | ✅ pass | 45000ms |
| 2 | `grep -c 'evo_stage' assets/data/units.ron` | 0 | ✅ pass — 12 | 50ms |

## Deviations

The two inline failing-parse test RON literals in units_ron.rs were updated to include valid evo_stage/evo_line/evolves_to fields. The plan mentioned updating `round_trip_unit_def` only, but the other two inline RON literals also needed the new mandatory fields to preserve their specific error assertions.

## Known Issues

none

## Files Created/Modified

- `src/combat/types.rs`
- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- `tests/bootstrap_spawn_composition.rs`
- `tests/roster_smoke.rs`
- `tests/follow_up_reentrancy.rs`
