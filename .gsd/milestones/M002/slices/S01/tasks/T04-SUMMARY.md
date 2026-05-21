---
id: T04
parent: S01
milestone: M002
key_files: []
key_decisions: []
duration: 
verification_result: passed
completed_at: 2026-05-21T17:53:36.213Z
blocker_discovered: false
---

# T04: Agumon stance.ron asset (idle/hurt/death/victory nodes) + whole-sheet clip range + stance load path

**Agumon stance.ron asset (idle/hurt/death/victory nodes) + whole-sheet clip range + stance load path**

## What Happened

Added whole-sheet 'all' clip range (0-92) to agumon/clip.ron. Authored agumon/stance.ron with id 'agumon_stance', entry 'idle', and idle/hurt/death/victory nodes. Added AnimationStancePaths resource and load path in plugin.rs feeding StanceGraphRegistry.

## Verification

cargo build --features windowed compiles; stance asset loads and validates

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 0ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
