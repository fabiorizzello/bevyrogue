---
estimated_steps: 11
estimated_files: 4
skills_used: []
---

# T01: Added pure energy-cap and deferred Tamer resource affordance queries

Expected `skills_used` frontmatter for executor: `test`, `verify-before-complete`.

Why: S05 must make non-executable resource systems queryable before UI work. Add pure, DSL/query-backed resource affordance helpers rather than any UI hardcoding. The existing S04 query already has `ResourceKind::{TamerGauge,TamerCommand,EnergyCap}` and reason codes, but lacks public helpers/data for actual Energy cap budgets and Tamer/Child command declarations.

Do:
1. Extend `UnitQuerySnapshot` in `src/combat/action_query.rs` with Energy cap budget/counter fields needed to answer secondary/external Energy cap queries deterministically. Choose names that preserve meaning, e.g. `energy_secondary_gained` and `energy_external_gained`, and update existing fixtures in `tests/action_affordance_query.rs` with zero defaults.
2. Add pure query helpers in `src/combat/action_query.rs` for Energy caps and deferred Tamer/Child resources. At minimum provide queryable details for Energy cap remaining/exhausted, Tamer Gauge, Tamer Commands (`Data Scan` 20, `Emergency Guard` 50, `Retreat` 100), and Child Tamer Gauge boost. These should return `ResourceAffordanceDetail` using `ResourceKind`, `ResourceStatus`, and `LegalityReasonCode`, not display strings.
3. Add tests to `tests/action_affordance_query.rs` proving Energy cap detail is enabled when budget remains, disabled with `EnergyCapReached` when exhausted or requested exceeds remaining, and Tamer/Child declarations are deferred/hidden with `TamerGaugeDeferred` / `TamerCommandDeferred` and required costs where known.
4. Update docs only where needed so `docs/skill_legality_contract.md` and `docs/combat_ui_readiness_gap_matrix.md` describe S05's contract accurately: Tamer/Child systems are declared/deferred; Energy cap state is queryable.

Failure Modes (Q5): incomplete snapshots should default to conservative zero-used cap budgets in tests or force explicit construction; do not panic in public query helpers for missing non-critical resource state.
Load Profile (Q6): pure query is O(number of declared commands) plus existing snapshot scanning; it must not allocate unbounded data or inspect ECS.
Negative Tests (Q7): include exhausted cap, partial remaining cap, and deferred command declarations that are not enabled/executable.

Done when: the pure query surface can answer all S05 resource-affordance questions with machine-readable reason codes, and `cargo test-dev --test action_affordance_query` passes.

## Inputs

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
- `docs/skill_legality_contract.md`
- `docs/combat_ui_readiness_gap_matrix.md`

## Expected Output

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
- `docs/skill_legality_contract.md`
- `docs/combat_ui_readiness_gap_matrix.md`

## Verification

cargo test-dev --test action_affordance_query

## Observability Impact

Pure `ResourceAffordanceDetail` output becomes the diagnostic surface for cap/deferred resource UI truthfulness; future agents can inspect test failures and exact reason-code assertions.
