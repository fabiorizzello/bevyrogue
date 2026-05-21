---
id: T05
parent: S07
milestone: M002
key_files:
  - tests/digimon_kits/holy_support_roster_contract.rs
key_decisions:
  - Updated the stale Holy Support roster contract to assert the new split contract explicitly: Agumon is opted into owner-keyed ult gauge metadata, while Gabumon remains metadata-free as a legacy control.
duration: 
verification_result: mixed
completed_at: 2026-05-21T21:02:21.011Z
blocker_discovered: false
---

# T05: Updated the stale Holy Support roster contract to accept Agumon’s energy-backed ult metadata while keeping legacy Digimon metadata-free.

**Updated the stale Holy Support roster contract to accept Agumon’s energy-backed ult metadata while keeping legacy Digimon metadata-free.**

## What Happened

Verified the suite first instead of making speculative fixture edits. The integration run showed the actual regression was a stale roster contract in `tests/digimon_kits/holy_support_roster_contract.rs`: it still asserted that Agumon had empty `blueprint_metadata`, but S07 intentionally opted Agumon into `agumon -> ult_gauge=energy`. I replaced that assertion with a two-sided contract check: Agumon now must expose the owner-keyed energy-backed gauge metadata, and Gabumon must remain `blueprint_metadata`-empty to preserve backward compatibility for non-opted-in legacy Digimon. No runtime code changes were needed because the rest of the snapshot fixture sweep was already aligned with the new `UnitQuerySnapshot`/`units_data` shape.

## Verification

Ran `cargo test --features windowed` before and after the patch. The first run failed only at `holy_support_roster_contract::blueprint_metadata_remains_optional_for_backward_compatibility`, proving the remaining regression was a stale contract test rather than runtime fixture shape breakage. After updating that contract to assert Agumon opt-in plus Gabumon legacy emptiness, the full suite passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed` | 101 | ❌ fail — exposed stale Agumon metadata assertion in holy_support_roster_contract | 8361ms |
| 2 | `cargo test --features windowed` | 0 | ✅ pass — full integration suite green after contract update | 5092ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `tests/digimon_kits/holy_support_roster_contract.rs`
