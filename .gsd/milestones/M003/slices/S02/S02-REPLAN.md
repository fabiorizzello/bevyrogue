# S02 Replan

**Milestone:** M003
**Slice:** S02
**Blocker Task:** T02
**Created:** 2026-05-22T11:36:04.979Z

## Blocker Description

Manual K001 verification of the windowed bridge (after T02) surfaced three plan gaps that block the S02 visual demo ("smooth animation, damage on impact, both actors"). None were captured by any S01/S02 task: (1) no animation clock — advance_agumon_presentation stepped the player once per render frame (~60fps), so idle cycled in ~0.1s and attack clips flashed in <0.15s, making "smooth animation" unverifiable; (2) cross-caster animation bug — the kernel cue barrier is global and CueBarrierStatus did not expose the caster, so both the ally and the dummy sprite entered Skill mode and animated an attack only one of them launched; (3) native 512px sprites filled the viewport, so a 4-per-team roster could not fit and the user could not assess on-screen layout. These are S01 (atlas-binding) infra gaps plus one bug, discovered only at the windowed surface which auto-mode cannot exercise (K001).

## What Changed

Added T04 to capture the visual-readiness fixes already implemented and verified headless during the manual K001 loop, bringing them inside the slice contract. T01/T02 (complete) and T03 (the Baby Flame / Baby Burner release-bridge activation, still pending) are unchanged. T04 sits before T03 in intent but the bridge work (T03) remains the final task of the slice.
