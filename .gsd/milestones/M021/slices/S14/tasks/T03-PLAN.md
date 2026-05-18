---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Prove add-new-digimon isolation and capture cross-slice boundaries

Create or update a scripted add-new-digimon proof that demonstrates the intended extension boundary and records the cross-slice integration contracts that consume the earlier migration work. If the current code still needs shared edits, capture the smallest truthful scope and document the remaining gap explicitly.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S12/S12-SUMMARY.md`
- `.gsd/milestones/M021/slices/S08/S08-SUMMARY.md`
- `.gsd/milestones/M021/slices/S09/S09-SUMMARY.md`
- `.gsd/milestones/M021/slices/S10/S10-SUMMARY.md`
- `.gsd/milestones/M021/slices/S11/S11-SUMMARY.md`

## Expected Output

- `tests`
- `.gsd/milestones/M021/slices/S14/tasks/T03-SUMMARY.md`

## Verification

cargo test -- --nocapture add_new_digimon || true
cargo test -- --nocapture blueprint || true

## Observability Impact

Provides the missing extension-boundary and cross-slice integration evidence that milestone validation asked for.
