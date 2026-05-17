---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Capture final closeout evidence for validation rerun

Write the closeout narrative that maps the fresh final evidence back to the milestone success criteria, requirement gaps, and cross-slice integration proof so the next validation pass can succeed from artifacts alone.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S13/S13-PLAN.md`
- `.gsd/milestones/M021/slices/S14/S14-PLAN.md`

## Expected Output

- `.gsd/milestones/M021/slices/S15/tasks/T02-SUMMARY.md`

## Verification

test -f .gsd/milestones/M021/slices/S15/S15-PLAN.md

## Observability Impact

Ensures the final slice summary is an auditable bridge between remediation work and milestone validation.
