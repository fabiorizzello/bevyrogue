---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T01: Capture cast_id, UltInstant, and turn-pipeline proof

Audit existing tests and runtime paths for `cast_id`, `UltInstant`, and turn-phase ordering. Add or update focused integration tests so the current tree proves these deferred M021 contracts explicitly instead of relying on old slice roadmap claims. Capture exactly which context requirements each test discharges in the task summary.

## Inputs

- `.gsd/milestones/M021/M021-CONTEXT.md`
- `.gsd/milestones/M021/slices/S01/S01-SUMMARY.md`
- `.gsd/milestones/M021/slices/S03/S03-RESEARCH.md`

## Expected Output

- `tests`
- ` .gsd/milestones/M021/slices/S13/tasks/T01-SUMMARY.md`

## Verification

cargo test -- --nocapture cast_id || true
cargo test -- --nocapture ult_instant || true
cargo test -- --nocapture turn_phase || true

## Observability Impact

Makes foundational event and turn-order invariants visible through focused tests and task evidence.
