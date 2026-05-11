---
id: T01
parent: S05
milestone: M012
key_files:
  - src/combat/action_query.rs
  - tests/action_affordance_query.rs
  - docs/skill_legality_contract.md
  - docs/combat_ui_readiness_gap_matrix.md
key_decisions:
  - Keep energy-cap math snapshot-driven via explicit per-unit counters rather than ECS inspection.
  - Expose Tamer Gauge and Tamer Commands as separate deferred query declarations so UI consumers cannot infer executability from action legality.
  - Represent Child gauge boost under the same deferred Tamer Gauge vocabulary until the mechanic is implemented.
duration: 
verification_result: passed
completed_at: 2026-05-01T07:46:04.742Z
blocker_discovered: false
---

# T01: Added pure energy-cap and deferred Tamer resource affordance queries

**Added pure energy-cap and deferred Tamer resource affordance queries**

## What Happened

Extended `UnitQuerySnapshot` with per-round energy-cap counters (`energy_secondary_gained`, `energy_external_gained`) so resource queries can compute remaining budget deterministically from snapshot state. Added pure query helpers in `src/combat/action_query.rs` for `EnergyCap` remaining/exhausted checks plus deferred Tamer Gauge and Tamer Command declarations (including the known 20/50/100 command costs), keeping all outputs machine-readable via `ResourceKind`, `ResourceStatus`, and `LegalityReasonCode` instead of display strings. Updated `tests/action_affordance_query.rs` to cover remaining-cap success, exhausted/over-budget failure, and deferred command declarations, and refreshed the legality-contract docs so they now state that Energy-cap state is queryable and Tamer/Child mechanics stay deferred/query-only until later slices.

## Verification

Fresh verification completed after the last code change: `cargo test-dev --test action_affordance_query` passed with 22 tests green, including the new energy-cap and Tamer declaration assertions. I also ran the doc-regression checks `cargo test-dev --test skill_legality_contract_docs --test ui_readiness_gap_matrix_docs`; both doc suites passed (10/10 and 7/7).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_query` | 0 | ✅ pass | 5200ms |
| 2 | `cargo test-dev --test skill_legality_contract_docs --test ui_readiness_gap_matrix_docs` | 0 | ✅ pass | 5800ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
- `docs/skill_legality_contract.md`
- `docs/combat_ui_readiness_gap_matrix.md`
