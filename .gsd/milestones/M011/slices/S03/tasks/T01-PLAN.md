---
estimated_steps: 20
estimated_files: 7
skills_used: []
---

# T01: Add EvoStage enum + EvoLineId type + migrate all UnitDef sites and RON

Add the EvoStage 7-variant enum and EvoLineId(String) newtype to src/combat/types.rs. Add three new mandatory fields (evo_stage, evo_line, evolves_to) to UnitDef in src/data/units_ron.rs. Then fix every compilation error: update all 12 entries in assets/data/units.ron, taichi_def() in bootstrap.rs, 5 UnitDef literals in tests/bootstrap_spawn_composition.rs, 2 in tests/roster_smoke.rs, 1 Impmon literal in tests/follow_up_reentrancy.rs, and 2 inline test literals in src/data/units_ron.rs (round_trip_unit_def, parse_canonical_units_ron assertions). This is one task because T01 alone would leave the project uncompilable — the compiler errors from adding mandatory fields guide every change site.

## Steps

1. In `src/combat/types.rs`, add after the SkillId struct:
   - `EvoStage` enum: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]` with variants BabyI, BabyII, Child, Adult, Perfect, Ultimate, SuperUltimate
   - `EvoLineId` newtype: `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] pub struct EvoLineId(pub String);` — NOT Copy (wraps String)
2. In `src/data/units_ron.rs`, add to UnitDef struct after `speed: i32`: `evo_stage: EvoStage`, `evo_line: EvoLineId`, `evolves_to: Vec<UnitId>`. Add `EvoStage, EvoLineId` to the use statement from types.
3. Update `assets/data/units.ron` — add three fields to each of the 12 entries. Mapping: Agumon/Gabumon/Dorumon/Renamon/Patamon/Tentomon = Child; Greymon/Garurumon/Kabuterimon/Kyubimon/DORUgamon/Angemon = Adult. Evo lines: agumon_line, gabumon_line, dorumon_line, renamon_line, patamon_line, tentomon_line. Each Child's evolves_to points to its Adult's UnitId; Adults have empty vec.
4. Update `src/combat/bootstrap.rs:taichi_def()` — add evo_stage: EvoStage::Child, evo_line: EvoLineId("tamer".into()), evolves_to: vec![]. Taichi is a human commander; Child + "tamer" line is a sentinel convention.
5. Update all 5 UnitDef literals in `tests/bootstrap_spawn_composition.rs` (lines 30, 53, 76, 99, 122) with evo_stage: EvoStage::Child, evo_line: EvoLineId("test".into()), evolves_to: vec![]
6. Update 2 UnitDef literals in `tests/roster_smoke.rs` (lines 65, 87) similarly.
7. Update the Impmon UnitDef literal in `tests/follow_up_reentrancy.rs` (line 287) similarly.
8. Update the round_trip_unit_def test literal in `src/data/units_ron.rs` with evo_stage/evo_line/evolves_to fields.
9. Run `cargo test --no-fail-fast` — all binaries must compile and pass.

## Must-Haves

- [ ] EvoStage has exactly 7 variants with JP naming (BabyI, BabyII, Child, Adult, Perfect, Ultimate, SuperUltimate)
- [ ] EvoLineId wraps String, derives Clone but NOT Copy
- [ ] UnitDef has evo_stage, evo_line, evolves_to as mandatory fields (no serde default)
- [ ] All 12 units.ron entries have correct evo_stage/evo_line/evolves_to
- [ ] All UnitDef construction sites in src/ and tests/ compile
- [ ] cargo test --no-fail-fast green

## Inputs

- ``src/combat/types.rs` — existing type definitions (DamageTag, Attribute, UnitId, SkillId)`
- ``src/data/units_ron.rs` — UnitDef struct definition and inline tests`
- ``assets/data/units.ron` — 12 unit entries to migrate`
- ``src/combat/bootstrap.rs` — taichi_def() at line 155`
- ``tests/bootstrap_spawn_composition.rs` — 5 UnitDef literals at lines 30, 53, 76, 99, 122`
- ``tests/roster_smoke.rs` — 2 UnitDef literals at lines 65, 87`
- ``tests/follow_up_reentrancy.rs` — Impmon UnitDef literal at line 287`

## Expected Output

- ``src/combat/types.rs` — EvoStage enum and EvoLineId newtype added`
- ``src/data/units_ron.rs` — UnitDef struct gains evo_stage, evo_line, evolves_to; round_trip test updated`
- ``assets/data/units.ron` — all 12 entries annotated with evo_stage/evo_line/evolves_to`
- ``src/combat/bootstrap.rs` — taichi_def() has sentinel evo fields`
- ``tests/bootstrap_spawn_composition.rs` — 5 literals updated`
- ``tests/roster_smoke.rs` — 2 literals updated`
- ``tests/follow_up_reentrancy.rs` — Impmon literal updated`

## Verification

cargo test --no-fail-fast 2>&1 | grep 'test result' && grep -c 'evo_stage' assets/data/units.ron
