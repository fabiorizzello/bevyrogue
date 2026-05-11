# S03: EvoStage 7-stage schema (JP) + RON migration — UAT

**Milestone:** M011
**Written:** 2026-04-27T15:30:04.177Z

# S03: EvoStage 7-stage schema (JP) + RON migration — UAT

**Milestone:** M011
**Written:** 2026-04-27

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: The core delivery is a data schema and asset migration. Automated tests provide definitive proof of parsing logic and data integrity.

## Preconditions

- Codebase compiles without errors (mandatory fields enforced).
- `assets/data/units.ron` contains 12 unit entries.

## Smoke Test

- Run `cargo test --no-fail-fast -p bevyrogue -- parse_canonical_units_ron --exact`
- **Expected:** Test passes, confirming all 12 units parse correctly with their new evolutionary metadata.

## Test Cases

### 1. Mandatory Field Enforcement (Fail-Loud)

1. Attempt to parse a RON string missing the `evo_stage` field into a `UnitDef`.
2. **Expected:** Parsing fails with a "missing field `evo_stage`" error. (Verified by `missing_evo_stage_fails_to_parse` test).

### 2. Evolutionary Data Integrity

1. Load the canonical roster.
2. Check that Agumon (id 1) has `evo_stage: Child` and `evolves_to: [UnitId(12)]`.
3. Check that Greymon (id 12) has `evo_stage: Adult` and `evolves_to: []`.
4. Check that both share the same `evo_line: "agumon_line"`.
5. **Expected:** All assertions hold true. (Verified by extended `parse_canonical_units_ron` test).

## Failure Signals

- Compilation errors in `UnitDef` construction sites.
- `units.ron` failing to parse during bootstrap.
- `missing_evo_stage_fails_to_parse` test failing to catch a missing field.

## Not Proven By This UAT

- Actual evolutionary transitions during combat (mechanic not yet implemented).
- Correctness of future stages (Perfect, Ultimate, etc.) beyond the schema level.

