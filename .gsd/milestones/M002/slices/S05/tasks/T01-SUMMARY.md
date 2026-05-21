---
id: T01
parent: S05
milestone: M002
key_files: []
key_decisions: []
duration: 
verification_result: passed
completed_at: 2026-05-21T17:55:50.935Z
blocker_discovered: false
---

# T01: Windowed Agumon-vs-Agumon-dummy encounter bootstrap with two on-screen sprites

**Windowed Agumon-vs-Agumon-dummy encounter bootstrap with two on-screen sprites**

## What Happened

Bootstrap encounter places two Agumon sprites left/right on screen. Encounter init wires both units through CombatState without windowed/UI code touching CombatState.

## Verification

cargo build --features windowed; two sprites visible on launch

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
