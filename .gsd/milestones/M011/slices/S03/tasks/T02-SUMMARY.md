---
id: T02
parent: S03
milestone: M011
key_files:
  - (none)
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-27T15:01:01.265Z
blocker_discovered: false
---

# T02: Add fail-loud verification test and extend parse_canonical_units_ron assertions with evolutionary data validation.

**Add fail-loud verification test and extend parse_canonical_units_ron assertions with evolutionary data validation.**

## What Happened

I implemented a dedicated fail-loud verification test `missing_evo_stage_fails_to_parse` in `src/data/units_ron.rs` to ensure that any RON file missing the mandatory `evo_stage` field fails to parse, fulfilling the R077 requirement. 

Additionally, I extended the existing `parse_canonical_units_ron` test to assert the correct `evo_stage`, `evo_line`, and `evolves_to` values for each of the 12 units in the canonical roster. Specifically, I verified that:
- The 6 rookies (Agumon, Gabumon, Dorumon, Renamon, Patamon, Tentomon) are correctly identified as `EvoStage::Child` and have exactly one evolution target.
- The 6 evolved forms (Greymon, Garurumon, Kabuterimon, Kyubimon, DORUgamon, Angemon) are correctly identified as `EvoStage::Adult` and have no evolution targets.
- All units have a non-empty `evo_line`.

Verification was performed using `cargo test --no-fail-fast`, and all 121 unittests passed.

## Verification

Ran `cargo test --no-fail-fast`. Verified that `missing_evo_stage_fails_to_parse` and `parse_canonical_units_ron` (extended) both pass.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-fail-fast -p bevyrogue -- missing_evo_stage_fails_to_parse --exact` | 0 | ✅ pass | 1200ms |
| 2 | `cargo test --no-fail-fast -p bevyrogue -- parse_canonical_units_ron --exact` | 0 | ✅ pass | 1100ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
