---
id: T03
parent: S01
milestone: M015
key_files:
  - docs/m015_failure_ledger.md
key_decisions:
  - Classified the missing M013 files as a closure/artifact gap, not a gameplay regression.
  - Kept the CLI issue owned by S05 because `src/bin/combat_cli.rs` still uses a relative asset path and the runtime smoke still panics.
duration: 
verification_result: passed
completed_at: 2026-05-08T10:58:08.322Z
blocker_discovered: false
---

# T03: Recorded the M013 closure-artifact gap and cwd-sensitive combat CLI blocker in the M015 failure ledger.

**Recorded the M013 closure-artifact gap and cwd-sensitive combat CLI blocker in the M015 failure ledger.**

## What Happened

I inspected the current M013 artifact paths in this worktree and confirmed that `.gsd/milestones/M013/M013-VALIDATION.md`, `.gsd/milestones/M013/MILESTONE-SUMMARY.md`, and `.gsd/milestones/M013/M013-CONTEXT.md` are all missing, so the remaining issue is a closure/artifact gap rather than a gameplay regression. I also inspected `src/bin/combat_cli.rs` and confirmed the startup path still reads `assets/data/units.ron` through `std::fs::read_to_string("assets/data/units.ron")`, which keeps the CLI harness cwd-sensitive. A fresh CLI smoke (`cargo run --bin combat_cli --quiet`) still exits with a Bevy `Message not initialized` panic, so deeper CLI proof remains owned by S05. I updated `docs/m015_failure_ledger.md` with a dedicated M013 artifact gap section and a clarified CLI gap row that names the relative asset path, the runtime symptom, and the downstream owner. No `.gsd` artifact repair was performed in this task; S06 remains the owner for closure packaging.

## Verification

Verified the artifact gap and CLI gap evidence directly from the worktree: the three M013 closure artifact paths are missing, the CLI source still hardcodes a relative `assets/data/units.ron` read, and `cargo run --bin combat_cli --quiet` still reproduces the Bevy runtime panic instead of proving the shared-surface flow. Verified the ledger update by grepping `docs/m015_failure_ledger.md` for `M013-VALIDATION`, `MILESTONE-SUMMARY`, `M013-CONTEXT`, `assets/data/units.ron`, and `src/bin/combat_cli.rs`.

Evidence rows were captured from a fresh verification digest and the CLI smoke run.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo run --bin combat_cli --quiet` | 101 | ✅ pass | 220ms |
| 2 | `gsd_exec verification digest: missing M013 artifact count + ledger grep coverage` | 0 | ✅ pass | 4ms |

## Deviations

None.

## Known Issues

M013 closure artifacts remain absent in this worktree; the CLI runtime still panics in Bevy before deeper proof, and the relative asset read in `src/bin/combat_cli.rs` remains cwd-sensitive.

## Files Created/Modified

- `docs/m015_failure_ledger.md`
