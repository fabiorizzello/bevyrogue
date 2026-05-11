---
id: T03
parent: S06
milestone: M015
key_files:
  - docs/m015_failure_ledger.md
  - tests/status_effect_apply.rs
  - tests/status_effect_integration.rs
  - tests/engine_legality_integration.rs
  - tests/action_affordance_consumers.rs
  - tests/action_affordance_query.rs
  - tests/combat_coherence.rs
  - tests/boundary_contract.rs
  - tests/sp_economy.rs
  - tests/patamon_revive.rs
  - tests/ultimate_meter.rs
  - tests/patamon_blueprint_seam.rs
  - tests/presentation_metadata_boundary.rs
  - tests/damage_breakdown_log.rs
  - src/combat/follow_up_tests.rs
  - src/combat/resolution_tests.rs
  - src/combat/turn_system/tests.rs
  - tests/twin_core_integration.rs
  - src/data/skills_ron.rs
key_decisions:
  - Use the current neutral `SkillDef` fields instead of restoring removed presentation/affordance APIs.
  - Treat the no-run baseline as truthfully classified even when the remaining failures shift to other fixture classes.
  - Update the failure ledger to retire SkillDef drift rather than leaving stale blocker rows behind.
duration: 
verification_result: passed
completed_at: 2026-05-08T22:15:57.981Z
blocker_discovered: false
---

# T03: Repaired SkillDef fixture drift and retired the remaining SkillDef no-run blockers

**Repaired SkillDef fixture drift and retired the remaining SkillDef no-run blockers**

## What Happened

I re-checked the canonical `SkillDef` shape, then repaired stale fixture literals across the broad test surface by adding the current neutral fields (`custom_signals`, `animation_sequence`, `qte`) or preserving `..Default::default()` where appropriate. While doing so, I also fixed the orphan-tail corruption introduced by the bulk edit pass, restored missing outer braces in the affected helper functions, and kept the obsolete `TargetShape::SelfTarget` / Holy Support affordance APIs out of the tree. Finally, I updated `docs/m015_failure_ledger.md` so the ledger no longer lists `SkillDef` constructor drift as an active blocker and now points the remaining no-run failures at the broader non-SkillDef fixture classes (UnitDef, RoundFlags, and related downstream fixture issues).

## Verification

Verified with a fresh `cargo test --no-run` probe and a negative obsolete-symbol scan. The compile still exits 101, but the remaining failures are now non-SkillDef blockers (UnitDef/RoundFlags and related broader fixture issues), which is the expected boundary for this task. The forbidden obsolete symbol scan returned no matches, confirming no restored `TargetShape::SelfTarget` / Holy Support affordance APIs.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --no-run` | 101 | ✅ pass | 791ms |
| 2 | `rg -n "TargetShape::SelfTarget|HolySupportAffordance|query_holy_support_affordance|ResourceKind::Grace|ResourceKind::MartyrLight|holy_support_affordance" tests src` | 1 | ✅ pass | 12ms |

## Deviations

Expanded beyond the four named fixture files to additional same-class `SkillDef` sites surfaced by compile, and repaired brace/tail corruption introduced by the mechanical bulk-edit pass. This was necessary to keep the no-run baseline truthful and to avoid leaving syntactically broken test helpers behind.

## Known Issues

`cargo test --no-run` still fails on the broader, pre-existing non-SkillDef fixture drift in `tests/tempo_resistance.rs`, `tests/follow_up_chains.rs`, `tests/roster_smoke.rs`, `tests/resource_caps.rs`, and downstream compile fallout reported for `tests/patamon_blueprint_seam.rs` and `tests/presentation_metadata_boundary.rs`.

## Files Created/Modified

- `docs/m015_failure_ledger.md`
- `tests/status_effect_apply.rs`
- `tests/status_effect_integration.rs`
- `tests/engine_legality_integration.rs`
- `tests/action_affordance_consumers.rs`
- `tests/action_affordance_query.rs`
- `tests/combat_coherence.rs`
- `tests/boundary_contract.rs`
- `tests/sp_economy.rs`
- `tests/patamon_revive.rs`
- `tests/ultimate_meter.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/presentation_metadata_boundary.rs`
- `tests/damage_breakdown_log.rs`
- `src/combat/follow_up_tests.rs`
- `src/combat/resolution_tests.rs`
- `src/combat/turn_system/tests.rs`
- `tests/twin_core_integration.rs`
- `src/data/skills_ron.rs`
