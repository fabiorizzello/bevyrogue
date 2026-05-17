---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Prove strict boot validation for invalid timeline ids

Exercise boot-time timeline validation directly: prove a bad registry reference fails at `App::finish()` with a deterministic test or focused harness, and align the resulting evidence with the M021 strict-boot-validation criterion.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S02/tasks/T05-SUMMARY.md`

## Expected Output

- `tests`
- `.gsd/milestones/M021/slices/S13/tasks/T03-SUMMARY.md`

## Verification

cargo test -- --nocapture timeline_refs || true
cargo test -- --nocapture boot_validation || true

## Observability Impact

Makes unresolved timeline ids a current, testable boot failure instead of a historical claim.
