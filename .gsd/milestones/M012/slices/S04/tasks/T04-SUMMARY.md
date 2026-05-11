---
id: T04
parent: S04
milestone: M012
key_files:
  - src/combat/action_query.rs
  - src/combat/toughness.rs
  - tests/action_affordance_query.rs
key_decisions:
  - Store real `Toughness` snapshots on the pure query input so the query can call `visible_toughness` directly instead of duplicating team-only visibility logic.
  - Expose per-target `toughness_view` plus `toughness_reason` so callers can distinguish visible enemy bars from hidden ally/zero-max bars and hidden/deferred mechanics.
duration: 
verification_result: mixed
completed_at: 2026-05-01T07:27:06.527Z
blocker_discovered: false
---

# T04: Added helper-backed toughness visibility and reason-bearing target toughness views to the pure action query.

**Added helper-backed toughness visibility and reason-bearing target toughness views to the pure action query.**

## What Happened

I extended the pure combat query so it now carries real `Toughness` snapshots on `UnitQuerySnapshot` and uses `exposes_toughness_affordance` / `visible_toughness` from `src/combat/toughness.rs` instead of re-implementing enemy-only rules in the query layer. `query_target_affordance` now returns per-target `toughness_view` and `toughness_reason` alongside legality, so implemented enemy bars surface full toughness details while ally bars and zero-max enemy bars stay hidden with `ToughnessEnemyOnly` and hidden/deferred skills suppress toughness exposure entirely.

I updated `tests/action_affordance_query.rs` to pin the public contract end to end: happy-path enemy toughness visibility, ally and zero-bar hiding, hidden self-only form-identity behavior, deferred unsupported row-shape behavior, and the existing legality matrix for active/unit/phase/SP/ultimate/target cases. The focused query suite, the required S02/S03 regression bundle, and the feature-gated windowed compile all passed after the last code change. The full `cargo test-dev` run still fails in pre-existing `tests/form_identity.rs` regressions unrelated to this task, which I recorded as a known issue rather than a task blocker.

## Verification

Verified with fresh post-change checks: `cargo test-dev --test action_affordance_query` passed 18/18; `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs` passed 3/3, 1/1, 2/2, and 10/10 respectively; `cargo check --features "dev windowed"` passed; `lsp diagnostics` on `src/combat/action_query.rs` and `tests/action_affordance_query.rs` reported no diagnostics. Full `cargo test-dev` still reports the pre-existing `tests/form_identity.rs` failures (8 failing tests), which are outside this slice.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_query` | 0 | ✅ pass | 3500ms |
| 2 | `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs` | 0 | ✅ pass | 3000ms |
| 3 | `cargo test-dev` | 101 | ❌ fail | 15000ms |
| 4 | `cargo check --features "dev windowed"` | 0 | ✅ pass | 2670ms |

## Deviations

None.

## Known Issues

`cargo test-dev` still fails in pre-existing `tests/form_identity.rs` regressions (Greymon/Garurumon/Kabuterimon/Kyubimon/Dorugamon form-identity expectations). These failures were not introduced by this task and remain outside the S04 query-contract scope.

## Files Created/Modified

- `src/combat/action_query.rs`
- `src/combat/toughness.rs`
- `tests/action_affordance_query.rs`
