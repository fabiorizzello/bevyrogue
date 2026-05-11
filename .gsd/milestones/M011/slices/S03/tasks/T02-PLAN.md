---
estimated_steps: 10
estimated_files: 1
skills_used: []
---

# T02: Add fail-loud verification test and extend parse_canonical_units_ron assertions

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

## Inputs

- ``src/data/units_ron.rs` — UnitDef struct with evo_stage/evo_line/evolves_to from T01`
- ``assets/data/units.ron` — migrated RON with all evo fields from T01`

## Expected Output

- ``src/data/units_ron.rs` — missing_evo_stage_fails_to_parse test added, parse_canonical_units_ron extended with evo_stage/evolves_to assertions`

## Verification

cargo test --no-fail-fast -p bevyrogue -- missing_evo_stage_fails_to_parse --exact 2>&1 | grep 'test result' && cargo test --no-fail-fast -p bevyrogue -- parse_canonical_units_ron --exact 2>&1 | grep 'test result'
