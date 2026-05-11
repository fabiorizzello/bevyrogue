---
id: T01
parent: S03
milestone: M012
key_files:
  - src/data/skills_ron.rs
  - src/combat/resolution_tests.rs
  - src/combat/turn_system/tests.rs
  - tests/target_shape_truthfulness.rs
  - tests/status_effect_apply.rs
  - tests/status_effect_integration.rs
  - tests/sp_economy.rs
  - tests/revive_semantics.rs
  - tests/patamon_revive.rs
  - tests/combat_coherence.rs
  - tests/encounter_e2e.rs
  - tests/event_stream.rs
  - tests/resource_caps.rs
  - tests/toughness_categories.rs
  - tests/toughness_enemy_only.rs
  - tests/damage_breakdown_log.rs
  - tests/status_accuracy.rs
  - tests/ultimate_meter.rs
  - tests/boundary_contract.rs
key_decisions:
  - Used explicit `SkillImplementation::Implemented` / `Deferred { reason: UnimplementedTargetShape }` metadata rather than inferring legality from effect shapes.
  - Modeled revive fixtures as `TargetSide::Ally` + `TargetLife::Ko` so the DSL states intent directly.
  - Kept runtime validation semantics unchanged in this task; the migration only made the schema and fixtures explicit.
duration: 
verification_result: passed
completed_at: 2026-04-30T21:51:02.235Z
blocker_discovered: false
---

# T01: Added first-class SkillDef targeting and implementation metadata and migrated Rust fixtures to the new schema.

**Added first-class SkillDef targeting and implementation metadata and migrated Rust fixtures to the new schema.**

## What Happened

Extended `src/data/skills_ron.rs` with serde-friendly targeting/legality types (`SkillTargeting`, `TargetSide`, `TargetLife`, `SelfTargetRule`, `SkillImplementation`, and `LegalityReasonCode`) and made `SkillDef` require explicit `targeting` and `implementation` fields with `deny_unknown_fields` on the schema structs. Migrated every Rust-side `SkillDef { ... }` literal in `src/` and `tests/` to declare explicit metadata: offensive fixtures now declare single-target enemy/live/forbid/implemented behavior, revive fixtures declare ally/KO targeting, and row/all-enemies truthfulness fixtures explicitly defer via `UnimplementedTargetShape`. Repaired the affected multi-line `use bevyrogue::data::{...}` imports during the bulk migration so the workspace compiles cleanly against the new required fields without changing runtime legality semantics in this slice.

## Verification

`cargo check --tests` passed with exit code 0. A structural scan across `src/` and `tests/` found 46 `SkillDef {` literals and 0 remaining blocks missing either `targeting:` or `implementation:`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --tests` | 0 | ✅ pass | 158ms |
| 2 | `python3 SkillDef metadata scan` | 0 | ✅ pass | 19ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `src/combat/resolution_tests.rs`
- `src/combat/turn_system/tests.rs`
- `tests/target_shape_truthfulness.rs`
- `tests/status_effect_apply.rs`
- `tests/status_effect_integration.rs`
- `tests/sp_economy.rs`
- `tests/revive_semantics.rs`
- `tests/patamon_revive.rs`
- `tests/combat_coherence.rs`
- `tests/encounter_e2e.rs`
- `tests/event_stream.rs`
- `tests/resource_caps.rs`
- `tests/toughness_categories.rs`
- `tests/toughness_enemy_only.rs`
- `tests/damage_breakdown_log.rs`
- `tests/status_accuracy.rs`
- `tests/ultimate_meter.rs`
- `tests/boundary_contract.rs`
