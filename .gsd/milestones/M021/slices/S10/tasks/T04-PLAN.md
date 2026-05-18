---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T04: Reconcile Dorumon predator runtime behavior with the generic blueprint transition contract

Follow up the T03 blocker by tracing why the final predator runtime transition now stops at `CapReached { cap: PreyLock }` instead of the previously expected applied prey-lock transition. Decide whether the correct contract is a test expectation change, a transition application fix, or owner-runtime sequencing adjustment; implement the minimal correction in the Dorumon owner/runtime path and update focused regression coverage so the predator runtime proof is green before broader observability work resumes.

## Inputs

- `.gsd/milestones/M021/slices/S10/tasks/T03-SUMMARY.md`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `tests/dorumon_predator_runtime.rs`

## Expected Output

- `tests/dorumon_predator_runtime.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `src/combat/kernel.rs`
- `src/combat/events.rs`

## Verification

cargo test --test dorumon_predator_runtime
cargo test --test event_stream

## Observability Impact

Validation snapshots and CLI proof output become the generic, owner-agnostic inspection surface for migrated blueprint mechanics.
