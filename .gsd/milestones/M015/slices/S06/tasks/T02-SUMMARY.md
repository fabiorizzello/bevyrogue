---
id: T02
parent: S06
milestone: M015
key_files:
  - docs/combat_ui_readiness_gap_matrix.md
  - docs/m015_failure_ledger.md
  - scripts/verify_m015_failure_ledger.py
key_decisions:
  - Kept `Cargo.toml` unchanged because the stale `battery_loop_resolution` declaration was not present.
  - Derived the readiness matrix from `docs/skill_legality_contract.md` so the UI/CLI boundary stays query-driven and non-authoritative.
  - Updated the ledger verifier to match the repaired ledger section/marker wording instead of forcing the docs back to the older open-gap phrasing.
duration: 
verification_result: passed
completed_at: 2026-05-08T22:02:45.216Z
blocker_discovered: false
---

# T02: Restored the UI readiness matrix doc and retired the stale docs gap in the failure ledger.

**Restored the UI readiness matrix doc and retired the stale docs gap in the failure ledger.**

## What Happened

I confirmed the manifest already had the kernel replacement coverage (`battery_loop_kernel`) and did not need a `Cargo.toml` edit. I then created `docs/combat_ui_readiness_gap_matrix.md` from the legality contract vocabulary so the UI/CLI readiness matrix now names R085, D053, D054, the `Implemented`/`ToFixNow`/`Deferred`/`Hidden` status set, the non-authority query boundary, and the required mechanic/reason-code phrases. After that I refreshed `docs/m015_failure_ledger.md` to record the docs artifact gap as resolved by T02 and synchronized `scripts/verify_m015_failure_ledger.py` with the new ledger structure and wording so the ledger verifier reflects the repaired closure state instead of the older open-gap phrasing.

## Verification

Verified the repaired doc contract with `cargo test --test ui_readiness_gap_matrix_docs` via `gsd_exec` (7/7 tests passed). Verified the manifest boundary still points at the replacement coverage with `grep -n "battery_loop_kernel" Cargo.toml` via `gsd_exec`. Verified the updated failure-ledger bookkeeping with `python3 scripts/verify_m015_failure_ledger.py` via `gsd_exec` after syncing the verifier to the new resolved-section wording.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test ui_readiness_gap_matrix_docs` | 0 | ✅ pass | 3698ms |
| 2 | `grep -n "battery_loop_kernel" Cargo.toml` | 0 | ✅ pass | 10ms |
| 3 | `python3 scripts/verify_m015_failure_ledger.py` | 0 | ✅ pass | 19ms |

## Deviations

No manifest edit was required because `battery_loop_kernel` was already present and `battery_loop_resolution` was already absent.

## Known Issues

None.

## Files Created/Modified

- `docs/combat_ui_readiness_gap_matrix.md`
- `docs/m015_failure_ledger.md`
- `scripts/verify_m015_failure_ledger.py`
