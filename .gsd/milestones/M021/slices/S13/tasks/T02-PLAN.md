---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Prove DryRun equals Execute parity

Add or tighten parity coverage so the same compiled timeline run produces the same pending intent stream in `DryRun` and `Execute` modes. Reuse the shared preview/timeline path where possible and record the exact invariant proven by the test.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S11/S11-SUMMARY.md`

## Expected Output

- `tests`
- `.gsd/milestones/M021/slices/S13/tasks/T02-SUMMARY.md`

## Verification

cargo test --test skill_preview -- --nocapture
cargo test -- --nocapture dry_run || true

## Observability Impact

Adds explicit parity evidence for I2 rather than inferring it from preview behavior.
