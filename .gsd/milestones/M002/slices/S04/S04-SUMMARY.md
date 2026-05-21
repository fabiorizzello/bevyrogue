---
id: S04
parent: M002
milestone: M002
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions: []
patterns_established:
  - (none)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-21T17:55:42.993Z
blocker_discovered: false
---

# S04: Baby Burner reactive detonate + flash VFX

**Baby Burner reactive detonate deterministic; flash VFX projected windowed without mutating combat state**

## What Happened

Delivered deterministic Baby Burner reactive detonate: lethal hit on Heated target detonates adjacent alive enemies for 8*heated_remaining Fire damage exactly once. Feature-gated flash indicator projects via CombatEvent only. R004 determinism intact.

## Verification

agumon_baby_burner_reactive passes; windowed build passes; no regressions

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

None.
