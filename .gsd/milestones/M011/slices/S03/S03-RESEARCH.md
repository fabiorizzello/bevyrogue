# S03 — Research

**Date:** 2026-04-27

## Summary

S03 adds the `EvoStage` enum and `evo_line`/`evolves_to` fields to `UnitDef`, migrates all 12 RON entries in `units.ron`, and ensures the parser fails loudly on legacy files missing the new fields. The work is straightforward: the design doc (combat_design.md sez. 15) fully specifies the enum and struct changes, the codebase has no existing `EvoStage` type to conflict with, and the pattern follows S02's atomic rename — add the type, add fields to the struct, update all construction sites, update RON, add a fail-loud test.

The blast radius is moderate but mechanical: every `UnitDef { ... }` literal in `src/` and `tests/` must gain the three new fields. There are ~10 such sites across 6 files. The RON migration is a 1:1 annotation of 12 existing entries with known evo_stage/evo_line/evolves_to values.

## Recommendation

Implement in three tasks: (1) add `EvoStage` enum + `EvoLineId` newtype to `types.rs`, add fields to `UnitDef`; (2) migrate `units.ron` and all `UnitDef` construction sites in `src/` and `tests/`; (3) add a dedicated fail-loud test for legacy RON missing `evo_stage`. This mirrors the S02 pattern (type first, data second, verification third) and keeps each task independently verifiable.

R077 is the sole requirement: enum with 7 JP-named stages, mandatory field, fail-loud on absence. MEM019 confirms JP naming convention (D040 lock).

## Implementation Landscape

### Key Files

- `src/combat/types.rs` — Add `EvoStage` enum (7 variants) and `EvoLineId(String)` newtype. Both need `Debug, Clone, Copy (EvoStage only), PartialEq, Eq, Hash, Serialize, Deserialize`.
- `src/data/units_ron.rs` — Add `evo_stage: EvoStage`, `evo_line: EvoLineId`, `evolves_to: Vec<UnitId>` to `UnitDef`. Update `round_trip_unit_def` test, `missing_identity_metadata_fails_to_parse` test (already covers missing required fields), and the `parse_canonical_units_ron` test. Add a new test `missing_evo_stage_fails_to_parse` for explicit fail-loud verification.
- `assets/data/units.ron` — Add `evo_stage`, `evo_line`, `evolves_to` to all 12 entries. Known mapping: Agumon/Gabumon/Dorumon/Renamon/Patamon/Tentomon = `Child`; Greymon/Garurumon/DORUgamon/Kyubimon/Kabuterimon/Angemon = `Adult`. Evo lines: `"agumon_line"`, `"gabumon_line"`, `"dorumon_line"`, `"renamon_line"`, `"patamon_line"`, `"tentomon_line"`. `evolves_to`: each Child points to its Adult's UnitId; Adults have empty vec.
- `src/combat/bootstrap.rs:155` — `taichi_def()` needs the three new fields. Taichi is a commander (human), not a Digimon — use a sentinel like `evo_stage: Child` (or a new `None` variant, but R077 says 7 stages so stick with the enum as-is; Taichi is a special case, `Child` with `evo_line: "tamer"` and empty `evolves_to`).
- `tests/bootstrap_spawn_composition.rs` — 5 `UnitDef` literals (lines 30, 53, 76, 99, 122) need the new fields.
- `tests/roster_smoke.rs` — 2 `UnitDef` literals (lines 65, 87) need the new fields.
- `tests/follow_up_reentrancy.rs` — 1 `UnitDef` literal (line 287, TestImpmon) needs the new fields.

### Build Order

1. **T01: Add types + update UnitDef struct** — Add `EvoStage` and `EvoLineId` to `types.rs`. Add three fields to `UnitDef` in `units_ron.rs`. This breaks compilation everywhere — that's intentional, the compiler surfaces all sites that need updating.
2. **T02: Migrate RON + all UnitDef construction sites** — Update `units.ron` (12 entries), `taichi_def()` in `bootstrap.rs`, all test UnitDef literals in `tests/` (8 sites across 3 files), and the inline tests in `units_ron.rs` (round_trip, missing_metadata). Compiler errors from T01 guide every change.
3. **T03: Fail-loud verification test** — Add `missing_evo_stage_fails_to_parse` test in `units_ron.rs` that attempts to parse a UnitDef RON string without `evo_stage` and asserts the parse fails. Also add an assertion in `parse_canonical_units_ron` that all units have explicit `evo_stage` values matching their known stage (Child for id<=11, Adult for id>=12). Run full suite.

### Verification Approach

- `cargo test --no-fail-fast` — all 28+ binaries green after migration.
- `grep -rn 'EvoStage' src/ tests/` confirms the enum is used in types, UnitDef, and tests.
- The fail-loud test (`missing_evo_stage_fails_to_parse`) confirms RON without `evo_stage` produces a parse error.
- `parse_canonical_units_ron` already validates roster integrity — extending it to check `evo_stage` values provides regression coverage.

## Constraints

- R077 specifies exactly 7 variants with JP naming (BabyI, BabyII, Child, Adult, Perfect, Ultimate, SuperUltimate). No EN aliases.
- MEM019 (D040 lock): JP naming in code and design doc; EN only in disambiguating comments.
- `UnitDef` uses `#[derive(Serialize, Deserialize)]` via serde — new fields without `#[serde(default)]` are automatically mandatory (fail-loud on absence). This is the desired behavior per R077.
- `EvoLineId` should be `String`-based (not `Copy`) to match the RON ergonomics of `UnitId(u32)` vs `EvoLineId("agumon_line")`.

## Common Pitfalls

- **Taichi (UnitId 0) is not a Digimon** — it's a human commander. Assign `evo_stage: Child`, `evo_line: EvoLineId("tamer")`, `evolves_to: []` as a sentinel. The evo_stage field is structurally required; the value is semantically inert for non-Digimon entities. An alternative is `Option<EvoStage>` but R077 says "mandatory on all UnitDef" — keep it non-optional and use a convention for non-Digimon.
- **`EvoLineId` is not `Copy`** because it wraps `String`. Don't derive `Copy` on it; derive `Clone` only. This differs from `EvoStage` which should be `Copy`.
- **Test literal count** — there are exactly 8 UnitDef literals in `tests/` plus 1 in `bootstrap.rs` plus 2 in `units_ron.rs` inline tests. Missing any one will be a compiler error, so the compiler is the safety net.

## Sources

- combat_design.md sez. 15 (EvoStage schema specification)
- R077 (requirement: 7-stage enum, mandatory field, fail-loud)
- MEM019 (JP naming convention, D040 lock)
- S02 Summary (confirms DamageTag type is stable, no Element references remain)