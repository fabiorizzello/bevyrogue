---
id: T04
parent: S02
milestone: M016
key_files:
  - .gsd/milestones/M016/slices/S02/S02-PLAN.md
key_decisions:
  - S02 must avoid adding `SkillCustomSignal::Dorumon` or any equivalent per-Digimon top-level signal enum in `src/data/skills_ron.rs`.
  - S02 should treat Dorumon/DORUgamon Predator Loop behavior as blueprint/plugin-owned and keep shared systems generic.
duration: 
verification_result: mixed
completed_at: 2026-05-09T13:55:08.538Z
blocker_discovered: true
---

# T04: Confirmed that S02's static enum/shared-mechanic plan is blocked by the captured plugin-boundary feedback.

**Confirmed that S02's static enum/shared-mechanic plan is blocked by the captured plugin-boundary feedback.**

## What Happened

Reviewed the captured user feedback against the current S02 plan. CAP-749a38e2 rejects per-Digimon static signal enum growth and asks for a more dynamic plugin-like registration model where removing a Digimon primarily removes its file/plugin. CAP-92aab67d further challenges shared mechanic ownership for BatteryLoop/PredatorLoop-style features. Those captures invalidate the planned `SkillCustomSignal::Dorumon` addition and the shared Predator Loop framing in the pending S02 tasks.

## Verification

Planning-only verification completed by comparing current S02 must-haves/tasks against CAP-749a38e2 and CAP-92aab67d; the planned Dorumon-specific enum variant and shared mechanic placement conflict with both captures.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `Compared CAP-749a38e2 and CAP-92aab67d to S02 task requirements; verdict: blocker confirmed for the current enum/shared-mechanic plan.` | -1 | unknown (coerced from string) | 0ms |

## Deviations

This is a planning-only blocker assessment inserted to persist the already-captured design blocker before structural replanning; no production code was changed.

## Known Issues

The current pending T01-T03 plan still describes static enum and shared Predator Loop surfaces and must be rewritten immediately.

## Files Created/Modified

- `.gsd/milestones/M016/slices/S02/S02-PLAN.md`
