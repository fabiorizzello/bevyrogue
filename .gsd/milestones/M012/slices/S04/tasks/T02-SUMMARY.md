---
id: T02
parent: S04
milestone: M012
key_files:
  - src/combat/action_query.rs
  - tests/action_affordance_query.rs
key_decisions:
  - Use `UnitId`-addressable snapshot lookup for target legality instead of skill IDs or side tables.
  - Return `Deferred(UnimplementedTargetShape)` for implemented non-single shapes so unsupported area targeting never falls through to single-target legality.
  - Keep toughness visibility as a per-target affordance alongside legality so later slices can inspect it without extra state reads.
duration: 
verification_result: mixed
completed_at: 2026-05-01T06:54:01.301Z
blocker_discovered: false
---

# T02: Added pure target-affordance evaluation with ID-based snapshot lookup and stable reason codes.

**Added pure target-affordance evaluation with ID-based snapshot lookup and stable reason codes.**

## What Happened

Implemented `query_target_affordance` and `query_all_target_affordances` in `src/combat/action_query.rs` as pure DSL-driven helpers that inspect `SkillDef.targeting` and `SkillDef.implementation` without hardcoded skill IDs. The query now resolves actor/target by `UnitId`, applies the required priority order for missing target, commander, self, side, life, damaged-HP, and unsupported-shape cases, and returns reason-coded `TargetStatus` values plus toughness visibility. I also expanded the combat query snapshot to carry ID-addressable units while retaining legacy acting/target fields for compatibility, and replaced the placeholder test with target-matrix coverage for offensive, revive, damaged-target, and deferred row/non-single behavior.

## Verification

Slice verification command passed: `cargo test-dev --test action_affordance_query`. A broader `cargo test-dev` run was also executed; it passed this task's target-affordance suite but surfaced unrelated pre-existing failures in `tests/form_identity.rs` (energy/form-identity regressions outside this slice).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_query` | 0 | ✅ pass | 4500ms |
| 2 | `cargo test-dev` | 101 | ❌ fail | 4800ms |

## Deviations

Extended `CombatQuerySnapshot` with an ID-addressable `units` list and `UnitQuerySnapshot.id` so the pure target query can resolve actor/target identities directly, while still falling back to legacy acting/target fields for older fixtures.

## Known Issues

`cargo test-dev` currently fails in unrelated `tests/form_identity.rs` cases (Greymon/Garurumon/Kabuterimon/Kyubimon/Dorugamon form-identity energy/toughness expectations). These failures were not introduced by this task and remain outside the target-affordance slice.

## Files Created/Modified

- `src/combat/action_query.rs`
- `tests/action_affordance_query.rs`
