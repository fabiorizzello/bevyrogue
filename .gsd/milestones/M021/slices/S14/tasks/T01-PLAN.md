---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T01: Prove HeadlessAuto and Windowed intent-stream parity

Add or tighten an explicit two-clock parity test that drives the same encounter or compiled timeline under HeadlessAuto and Windowed semantics, then asserts identical emitted intent streams and end-of-cast outcomes.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S03/S03-RESEARCH.md`
- `.gsd/milestones/M021/slices/S11/S11-SUMMARY.md`

## Expected Output

- `tests`
- `.gsd/milestones/M021/slices/S14/tasks/T01-SUMMARY.md`

## Verification

cargo test -- --nocapture windowed || true
cargo test -- --nocapture parity || true

## Observability Impact

Makes D026 visible through a direct parity test rather than compile-only evidence.
