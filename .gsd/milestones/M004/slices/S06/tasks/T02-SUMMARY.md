---
id: T02
parent: S06
milestone: M004
key_files:
  - scripts/capture-windowed-m004-vfx.sh
key_decisions:
  - Preserved the established human-only capture-helper pattern by using the `.cargo` `cargo winx` alias, teeing stdout/stderr into milestone-local `.gsd/.../uat-evidence`, and keeping an explicit K001 banner that forbids auto-mode execution.
duration: 
verification_result: passed
completed_at: 2026-05-25T20:35:22.278Z
blocker_discovered: false
---

# T02: Added a K001-bannered M004 windowed capture helper that logs `cargo winx` output into the S06 UAT evidence directory.

**Added a K001-bannered M004 windowed capture helper that logs `cargo winx` output into the S06 UAT evidence directory.**

## What Happened

Created `scripts/capture-windowed-m004-vfx.sh` by mirroring the existing M002 smoke-capture helper while adapting it to the M004/S06 VFX signoff flow. The new script derives `REPO_ROOT` from `BASH_SOURCE`, runs with `set -uo pipefail`, creates `.gsd/milestones/M004/slices/S06/uat-evidence`, writes a timestamped `windowed-vfx-*.log`, and tees stdout/stderr from `cargo winx` so the human-run windowed session preserves the existing playback/load diagnostics in a discoverable evidence artifact. The header comment and banner explicitly preserve K001 by stating that auto-mode must not invoke the script and that only a human operator should launch the windowed binary. After writing the file, I marked it executable and verified only its syntax/content without executing it.

## Verification

Ran the task-plan verification command against `scripts/capture-windowed-m004-vfx.sh`; it passed, confirming the script is executable, parses under `bash -n`, contains the K001 / `auto-mode must NOT invoke` banner, points at `.gsd/milestones/M004/slices/S06/uat-evidence`, and invokes `cargo winx`. Per K001, the script itself was not executed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -x scripts/capture-windowed-m004-vfx.sh && bash -n scripts/capture-windowed-m004-vfx.sh && grep -qi 'K001' scripts/capture-windowed-m004-vfx.sh && grep -q 'auto-mode must NOT invoke' scripts/capture-windowed-m004-vfx.sh && grep -q 'M004/slices/S06/uat-evidence' scripts/capture-windowed-m004-vfx.sh && grep -q 'winx' scripts/capture-windowed-m004-vfx.sh` | 0 | ✅ pass | 11ms |

## Deviations

None.

## Known Issues

Human windowed capture remains pending by design under K001; this task only adds the helper script and does not run the windowed binary.

## Files Created/Modified

- `scripts/capture-windowed-m004-vfx.sh`
