---
id: T04
parent: S05
milestone: M002
key_files: []
key_decisions: []
duration: 
verification_result: passed
completed_at: 2026-05-21T17:56:08.047Z
blocker_discovered: false
---

# T04: Per-hop kernel cue: visible Baby Flame loop iterations = kernel hop_index count

**Per-hop kernel cue: visible Baby Flame loop iterations = kernel hop_index count**

## What Happened

Baby Flame multi-hit loop iterates exactly the kernel hop_index count via per-hop kernel cues. loop-hop cue parity test passes. bouncing_fire_off_baseline unchanged.

## Verification

loop-hop cue parity test passes; bouncing_fire_off_baseline passes

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 0ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

None.
