---
id: S03
parent: M012
milestone: M012
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/data/skills_ron.rs
  - src/combat/resolution.rs
  - assets/data/skills.ron
  - .gsd/PROJECT.md
  - tests/skill_legality_contract_docs.rs
  - tests/revive_semantics.rs
  - tests/patamon_revive.rs
  - tests/target_shape_truthfulness.rs
key_decisions:
  - SkillDef owns targeting and implementation metadata; there is no sidecar legality registry.
  - Validation is split between serde parse errors for malformed data and semantic validation for contradictory but well-typed skills.
  - Resolution copies target shape from declared metadata rather than inferring it from effects.
patterns_established:
  - Stable legality reason codes are machine-facing identifiers shared by data, validation, and future query consumers.
  - Deferred/hidden semantics are explicit data, not special-case UI behavior.
  - The canonical skill catalog is now a contract artifact that can be regression-tested independently of the pure query API.
observability_surfaces:
  - none
drill_down_paths:
  - .gsd/milestones/M012/slices/S03/tasks/T01-SUMMARY.md
  - .gsd/milestones/M012/slices/S03/tasks/T02-SUMMARY.md
  - .gsd/milestones/M012/slices/S03/tasks/T03-SUMMARY.md
  - .gsd/milestones/M012/slices/S03/tasks/T04-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-04-30T22:05:08.562Z
blocker_discovered: false
---

# S03: S03: Skill DSL targeting and legality metadata

**Made SkillDef legality explicit in the DSL, migrated all canonical skills to declared targeting/implementation metadata, and made resolution trust that metadata for target-shape propagation.**

## What Happened

S03 turned the skill catalog into a first-class legality contract instead of an inference problem. `SkillDef` now carries explicit `targeting` and `implementation` metadata with stable reason codes, all 72 canonical entries in `assets/data/skills.ron` were migrated, and the loader now separates malformed RON from well-typed semantic contradictions with skill-id-specific errors.

The validation layer rejects contradictory declarations loudly: implemented skills are still limited to Single target shape, revive semantics must declare ally/KO/single correctly, and mixed damage+revive skills must use deferred/hidden metadata with `UnimplementedEffect`. On the runtime side, `resolve_action` now copies `ResolvedAction.target_shape` from `SkillDef.targeting.shape`, so target-shape truth is driven by declared DSL data rather than effect-shape inference. This preserves the S02 rejection path while making S03 the canonical source of truth for future affordance queries.

The slice also updated the broader test surface so direct Rust fixtures across combat, resource, revive, status, toughness, and ultimate tests all compile against the new required metadata. Final verification stayed green across the focused skill-book regression suite, revive/target-shape regressions, and the feature-gated windowed compile.

## Verification

Fresh verification in this workspace passed:
- `cargo test-dev skills_ron`
- `cargo test-dev --test target_shape_truthfulness --test skill_legality_contract_docs --test revive_semantics --test patamon_revive`
- `cargo check --features "dev windowed"`
- `grep -c "targeting:" assets/data/skills.ron` → 72
- `grep -c "implementation:" assets/data/skills.ron` → 72

Result summary: canonical skill parsing/validation passed, malformed/contradictory metadata tests passed, metadata-driven target-shape regressions passed, revive semantics remained green, and the windowed feature-gated compile succeeded.

## Requirements Advanced

- R084 — advanced by making legality/targeting explicit in the DSL, validating canonical data, and removing effect-shape inference as the source of truth.
- R085 — advanced by declaring truthful deferred/hidden affordances in data so future UI/query surfaces can avoid false claims.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

None.

## Files Created/Modified

- `src/data/skills_ron.rs` — 
- `src/combat/resolution.rs` — 
- `assets/data/skills.ron` — 
- `tests/skill_legality_contract_docs.rs` — 
- `tests/revive_semantics.rs` — 
- `tests/patamon_revive.rs` — 
- `tests/target_shape_truthfulness.rs` — 
- `src/combat/resolution_tests.rs` — 
- `src/combat/follow_up_tests.rs` — 
- `src/combat/turn_system/tests.rs` — 
- `tests/resource_caps.rs` — 
- `tests/sp_economy.rs` — 
- `tests/status_effect_apply.rs` — 
- `tests/status_effect_integration.rs` — 
- `tests/boundary_contract.rs` — 
- `tests/combat_coherence.rs` — 
- `tests/damage_breakdown_log.rs` — 
- `tests/encounter_e2e.rs` — 
- `tests/event_stream.rs` — 
- `tests/status_accuracy.rs` — 
- `tests/toughness_categories.rs` — 
- `tests/toughness_enemy_only.rs` — 
- `tests/ultimate_meter.rs` — 
- `.gsd/PROJECT.md` — 
