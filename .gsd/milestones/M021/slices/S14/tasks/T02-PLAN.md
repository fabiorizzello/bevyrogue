---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Prove blueprint isolation and no-Bevy shared boundaries

Audit blueprint modules for forbidden Bevy imports and the one-module one-register contract. Add a focused structural proof or test fixture that shows blueprint integration remains isolated to owner modules and shared registries.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S10/S10-SUMMARY.md`

## Expected Output

- `tests`
- `.gsd/milestones/M021/slices/S14/tasks/T02-SUMMARY.md`

## Verification

rg "use bevy" src/combat/blueprints/
rg -n "fn register\(" src/combat/blueprints/

## Observability Impact

Turns the blueprint isolation rule into current grep-backed evidence.
