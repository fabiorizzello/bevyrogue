---
estimated_steps: 3
estimated_files: 5
skills_used: []
---

# T02: Restore the UI readiness doc and manifest boundary

Retire the manifest/docs compile class without touching gameplay authority: remove obsolete `battery_loop_resolution` manifest declaration only if present, and create/repair the tracked UI readiness gap matrix doc from the existing legality contract. Expected executor skills: `api-design`, `tdd`, `verify-before-complete`, `write-docs`.

Steps: (1) confirm `battery_loop_kernel` remains replacement coverage; (2) satisfy `tests/ui_readiness_gap_matrix_docs.rs` with doc vocabulary (`R085`, `D053`, `D054`, `Implemented`, `ToFixNow`, `Deferred`, `Hidden`, non-authority/query-boundary phrases); (3) record the retired blocker in the ledger.

Must-haves: no docs test is loosened to pass; the doc states CLI/windowed legality consumers read DSL/query output and do not own skill-ID-specific legality. Failure modes/Q5-Q7: if docs assertions fail, update the doc from `docs/skill_legality_contract.md`; do not delete assertions or invent authority.

## Inputs

- ``docs/m015_failure_ledger.md` — T01 classification and closure constraints.`
- ``Cargo.toml` — manifest test-target state.`
- ``docs/skill_legality_contract.md` — reason-code and consumer-boundary source vocabulary.`
- ``tests/ui_readiness_gap_matrix_docs.rs` — executable doc contract.`

## Expected Output

- ``Cargo.toml` — obsolete target removed only if present.`
- ``docs/combat_ui_readiness_gap_matrix.md` — tracked readiness gap matrix contract doc.`
- ``docs/m015_failure_ledger.md` — manifest/docs blockers retired with evidence.`

## Verification

`cargo test --test ui_readiness_gap_matrix_docs` exits 0 via `gsd_exec`; `grep -q "battery_loop_kernel" Cargo.toml` confirms replacement coverage if manifest target edits occur.
