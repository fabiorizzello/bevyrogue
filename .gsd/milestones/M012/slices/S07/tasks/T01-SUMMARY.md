---
id: T01
parent: S07
milestone: M012
key_files:
  - src/combat/action_query.rs
  - src/combat/mod.rs
  - tests/action_affordance_consumers.rs
key_decisions:
  - Keep the engine-facing snapshot builder on the SP-bypass path and expose a separate explicit-SP helper for UI/CLI affordance snapshots.
  - Keep consumer selection helpers pure by filtering query output only; do not re-encode KO/team legality rules outside `query_action_affordance()`.
duration: 
verification_result: passed
completed_at: 2026-05-01T13:43:31.599Z
blocker_discovered: false
---

# T01: Added an explicit-SP affordance snapshot seam and query-backed target selection helpers.

**Added an explicit-SP affordance snapshot seam and query-backed target selection helpers.**

## What Happened

I split the combat affordance snapshot path into two public helpers: the original `build_snapshot_from_ecs()` continues to feed S06 engine parity with the SP-bypass snapshot, while `build_snapshot_from_ecs_with_sp()` lets UI/CLI consumers build preflight affordances from the real `SpPool.current` value. I also added pure selection helpers, `enabled_target_ids()` and `first_enabled_target_id()`, that only filter query output and do not re-encode KO, team, or target-side legality rules.

To lock the seam down, I added `tests/action_affordance_consumers.rs` with coverage for four contract points: revive cost shortfall with explicit SP, SP-bypass parity remaining separate for the engine-facing builder, disabled resource snapshots preserving KO/live/enemy target reason codes, and picking an enabled Basic target from query output instead of local team/KO assumptions. I also exported the new helpers from `src/combat/mod.rs` for downstream consumer wiring and recorded the architecture decision in memory for future combat-affordance work.

## Verification

Fresh after the last code change, `cargo test-dev --test action_affordance_consumers`, `cargo test-dev --test action_affordance_query`, and `cargo test-dev --test engine_legality_integration` all passed. The focused consumer suite validated the explicit-SP shortfall path, the engine bypass path, the preserved target reason codes, and the query-backed enabled Basic selection helper.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_consumers` | 0 | ✅ pass | 23300ms |
| 2 | `cargo test-dev --test action_affordance_query` | 0 | ✅ pass | 23300ms |
| 3 | `cargo test-dev --test engine_legality_integration` | 0 | ✅ pass | 23300ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/action_query.rs`
- `src/combat/mod.rs`
- `tests/action_affordance_consumers.rs`
