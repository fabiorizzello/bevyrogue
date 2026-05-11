# S03: EvoStage 7-stage schema (JP) + RON migration

**Goal:** Add EvoStage 7-stage enum (JP naming per R077/MEM019), EvoLineId newtype, and evo_stage/evo_line/evolves_to fields to UnitDef. Migrate all 12 RON entries and all UnitDef construction sites. Parser fails loudly on legacy RON missing evo_stage.
**Demo:** cargo test verde dopo migrazione; tutti gli UnitDef hanno evo_stage esplicito; test dedicato verifica fail-loud su file legacy

## Must-Haves

- ## Must-Haves
- EvoStage enum with exactly 7 JP-named variants: BabyI, BabyII, Child, Adult, Perfect, Ultimate, SuperUltimate (R077)
- EvoLineId(String) newtype for evo line grouping
- Three new mandatory fields on UnitDef: evo_stage: EvoStage, evo_line: EvoLineId, evolves_to: Vec<UnitId>
- All 12 units.ron entries annotated with correct evo_stage (Child for id 1-6, Adult for id 7-12), evo_line, and evolves_to
- All UnitDef literals in src/ and tests/ updated (taichi_def, 5 bootstrap_spawn_composition, 2 roster_smoke, 1 follow_up_reentrancy Impmon, 2 units_ron.rs inline tests)
- Fail-loud: RON missing evo_stage produces parse error (no serde default)
- Dedicated test proving fail-loud behavior
- cargo test --no-fail-fast passes all 28+ binaries
- ## Requirement Impact
- **Requirements touched**: R077 (primary delivery)
- **Re-verify**: Full test suite — UnitDef is touched everywhere
- **Decisions revisited**: none
- ## Proof Level
- This slice proves: contract
- Real runtime required: no
- Human/UAT required: no
- ## Verification
- `cargo test --no-fail-fast` — all 28+ binaries green
- `grep -rn 'EvoStage' src/combat/types.rs` confirms enum with 7 variants
- `grep -c 'evo_stage' assets/data/units.ron` returns 12 (one per unit)
- Dedicated `missing_evo_stage_fails_to_parse` test passes — proves fail-loud on legacy RON
- `parse_canonical_units_ron` extended to assert correct evo_stage values per unit

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Add EvoStage enum + EvoLineId type + migrate all UnitDef sites and RON** `est:45m`
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
  - Files: `src/combat/types.rs`, `src/data/units_ron.rs`, `assets/data/units.ron`, `src/combat/bootstrap.rs`, `tests/bootstrap_spawn_composition.rs`, `tests/roster_smoke.rs`, `tests/follow_up_reentrancy.rs`
  - Verify: cargo test --no-fail-fast 2>&1 | grep 'test result' && grep -c 'evo_stage' assets/data/units.ron

- [x] **T02: Add fail-loud verification test and extend parse_canonical_units_ron assertions** `est:20m`
  Add a dedicated test proving that RON without evo_stage fails to parse (R077 fail-loud requirement). Extend parse_canonical_units_ron to assert correct evo_stage values per unit. This validates R077's mandatory-field guarantee.

## Steps

1. In `src/data/units_ron.rs` tests module, add `missing_evo_stage_fails_to_parse` test: construct a RON string of a minimal UnitDef missing the evo_stage field, call `ron::from_str::<UnitDef>(&ron_str)`, assert it returns Err.
2. In the same tests module, extend `parse_canonical_units_ron` test: after the existing assertions, add a block that checks each unit's evo_stage against known values — Child for Agumon/Gabumon/Dorumon/Renamon/Patamon/Tentomon (ids 1-6), Adult for Greymon/Garurumon/Kabuterimon/Kyubimon/DORUgamon/Angemon. Also assert each unit has a non-empty evo_line and that Child units have exactly one evolves_to entry while Adult units have zero.
3. Run `cargo test --no-fail-fast` — all binaries green, including the two new/extended tests.

## Must-Haves

- [ ] missing_evo_stage_fails_to_parse test exists and passes
- [ ] parse_canonical_units_ron asserts correct evo_stage per unit
- [ ] parse_canonical_units_ron asserts Child units have evolves_to.len() == 1, Adult units have 0
- [ ] cargo test --no-fail-fast green
  - Files: `src/data/units_ron.rs`
  - Verify: cargo test --no-fail-fast -p bevyrogue -- missing_evo_stage_fails_to_parse --exact 2>&1 | grep 'test result' && cargo test --no-fail-fast -p bevyrogue -- parse_canonical_units_ron --exact 2>&1 | grep 'test result'

## Files Likely Touched

- src/combat/types.rs
- src/data/units_ron.rs
- assets/data/units.ron
- src/combat/bootstrap.rs
- tests/bootstrap_spawn_composition.rs
- tests/roster_smoke.rs
- tests/follow_up_reentrancy.rs
