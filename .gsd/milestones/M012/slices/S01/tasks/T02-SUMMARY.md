---
id: T02
parent: S01
milestone: M012
key_files:
  - docs/skill_legality_contract.md
  - tests/skill_legality_contract_docs.rs
key_decisions:
  - (none)
duration: 
verification_result: passed
completed_at: 2026-04-30T17:43:20.597Z
blocker_discovered: false
---

# T02: Added the R084 skill legality contract and executable doc guard.

**Added the R084 skill legality contract and executable doc guard.**

## What Happened

Created `docs/skill_legality_contract.md` as a fresh-reader contract for downstream M012 implementation slices. The document defines goals/non-goals, data ownership, pure query inputs, the four status shapes, stable reason-code families, target/resource/implementation semantics, engine preflight parity, and consumer rules forbidding UI-specific or skill-ID-specific legality tables. Added `tests/skill_legality_contract_docs.rs` to statically include the tracked markdown and fail with targeted messages when required statuses, reason codes, R084/D053 links, engine parity language, query-source rules, or placeholder-free documentation are missing.

## Verification

Verified the targeted T02 contract with `cargo test --test skill_legality_contract_docs`: 10 tests passed, exit code 0. Verified the S01 executable-doc set with `cargo test --test ui_readiness_gap_matrix_docs --test skill_legality_contract_docs`: 17 total tests passed across both doc guards, exit code 0. Existing compiler warnings are unchanged and unrelated to this documentation/test-only task.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test skill_legality_contract_docs` | 0 | ✅ pass | 200ms |
| 2 | `cargo test --test ui_readiness_gap_matrix_docs --test skill_legality_contract_docs` | 0 | ✅ pass | 210ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `docs/skill_legality_contract.md`
- `tests/skill_legality_contract_docs.rs`
