---
id: T01
parent: S01
milestone: M012
key_files:
  - docs/combat_ui_readiness_gap_matrix.md
  - tests/ui_readiness_gap_matrix_docs.rs
key_decisions:
  - Classified larger non-executable gameplay systems such as Tamer Gauge/Commands, Child Tamer Gauge boost, and enemy counterplay/telegraphs as Deferred/Hidden declarations rather than executable UI affordances, consistent with D054.
duration: 
verification_result: mixed
completed_at: 2026-04-30T17:38:31.336Z
blocker_discovered: false
---

# T01: Added the R085 combat UI-readiness gap matrix and executable doc guard.

**Added the R085 combat UI-readiness gap matrix and executable doc guard.**

## What Happened

Created the tracked gap matrix for R085 under docs/, synthesizing the S01 research findings into a fresh-reader contract for later M012 implementation slices. The document links R085, D053, and D054; defines the Implemented/ToFixNow/Deferred/Hidden vocabulary; states the hard boundary against CLI/windowed skill-ID-specific legality rules; and classifies all required UI-affecting mechanics with downstream slices and contract notes. Added an integration-style doc test that statically includes the tracked markdown and rejects missing requirement/decision links, missing statuses, missing mechanics, missing reason examples, missing boundary text, and placeholder text.

## Verification

Verified the T01 target with `cargo test --test ui_readiness_gap_matrix_docs`: 7 tests passed, exit code 0. Also ran the slice-level commands from S01: `cargo test --test skill_legality_contract_docs` and `cargo test --test ui_readiness_gap_matrix_docs --test skill_legality_contract_docs` currently exit 101 because T02 has not yet created the skill-legality contract doc/test target; this is expected for the first task in the slice and not a T01 failure.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test ui_readiness_gap_matrix_docs` | 0 | ✅ pass | 22733ms |
| 2 | `cargo test --test skill_legality_contract_docs` | 101 | ❌ fail (expected intermediate state: T02 target not created yet) | 147ms |
| 3 | `cargo test --test ui_readiness_gap_matrix_docs --test skill_legality_contract_docs` | 101 | ❌ fail (expected intermediate state: T02 target not created yet) | 148ms |

## Deviations

None.

## Known Issues

S01 slice-level verification remains partial until T02 creates `docs/skill_legality_contract.md` and `tests/skill_legality_contract_docs.rs`.

## Files Created/Modified

- `docs/combat_ui_readiness_gap_matrix.md`
- `tests/ui_readiness_gap_matrix_docs.rs`
