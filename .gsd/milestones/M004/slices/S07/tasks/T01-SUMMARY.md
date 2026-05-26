---
id: T01
parent: S07
milestone: M004
key_files:
  - .gsd/milestones/M004/slices/S04/verify_s04_docs.py
key_decisions:
  - Updated the stale S04 guard to track the current `projectile_on_expire_chains_the_impact_then_flash_fan` windowed-only proof token.
  - Kept S04 scope/boundary checks intact while removing only the overly-specific pending-header requirement that would conflict with S05/S06 superseding closeout language.
duration: 
verification_result: passed
completed_at: 2026-05-25T21:02:19.312Z
blocker_discovered: false
---

# T01: Repaired the S04 documentation guard to follow the current Baby Flame windowed proof token and tolerate S05/S06 superseding closeout wording.

**Repaired the S04 documentation guard to follow the current Baby Flame windowed proof token and tolerate S05/S06 superseding closeout wording.**

## What Happened

Read the existing S04 documentation checker plus the current S04 scope/boundary artifacts and the referenced proof tests. Confirmed the checker was stale because it still required the pre-S06 windowed-only token `projectile_on_expire_chains_the_impact_fan`, while the actual proof surface now uses `projectile_on_expire_chains_the_impact_then_flash_fan` to model the authored `impact -> impact_flash` chain. Updated only the checker: replaced the stale test-token assertion with the current test name and relaxed the exact `### Pending S05` / `### Pending S06` header checks so the historical S04 docs can continue to validate even if later closeout wording points to S05/S06 superseding artifacts instead of preserving those exact pending subheaders. All other S04 scope, boundary, and dependency assertions were left intact.

## Verification

Ran `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py` after the edit; it exited 0 and printed `OK: S04 validation docs, proof references, and S03 dependency metadata are consistent.`

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `python3 .gsd/milestones/M004/slices/S04/verify_s04_docs.py` | 0 | ✅ pass | 15ms |

## Deviations

Relaxed the exact `### Pending S05` / `### Pending S06` header assertions so the historical S04 guard can tolerate later doc wording that references S05/S06 closeout artifacts instead of freezing those items as permanently pending.

## Known Issues

None.

## Files Created/Modified

- `.gsd/milestones/M004/slices/S04/verify_s04_docs.py`
