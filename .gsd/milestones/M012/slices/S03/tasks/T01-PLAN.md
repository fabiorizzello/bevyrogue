---
estimated_steps: 74
estimated_files: 20
skills_used: []
---

# T01: Introduce SkillDef targeting metadata and migrate Rust fixtures

---
estimated_steps: 5
estimated_files: 21
skills_used:
  - tdd
  - verify-before-complete
---

Add the explicit DSL types that S03 needs, attach them to `SkillDef`, and update every Rust-side `SkillDef { ... }` construction site so the codebase compiles against the new required metadata before the canonical RON migration begins.

Steps:
1. In `src/data/skills_ron.rs`, add serde-friendly metadata types: `SkillTargeting`, `TargetSide`, `TargetLife`, `SelfTargetRule`, `SkillImplementation`, and a stable `LegalityReasonCode` enum with at least the reason codes needed by this slice (`UnimplementedTargetShape`, `UnimplementedEffect`, and target-side/life codes used by validation tests). Use `#[serde(deny_unknown_fields)]` on `SkillDef` and metadata structs.
2. Add required `targeting: SkillTargeting` and `implementation: SkillImplementation` fields to `SkillDef`; derive `Debug`, `Clone`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize` consistently with existing types.
3. Add small constructor/helper functions in test modules where useful, but do not introduce a sidecar legality registry. Every fixture must declare metadata that matches its intended current behavior.
4. Update all Rust fixture literals currently found by `rg -l "SkillDef \\{" src tests` so damage skills declare `shape: Single`, `side: Enemy`, `life: Alive`, `self_rule: Forbid`, `implementation: Implemented`; revive fixtures declare `side: Ally`, `life: Ko`; existing Row/AoE truthfulness fixtures declare Row metadata with deferred implementation unless the test is specifically proving mismatch validation later.
5. Keep existing behavior assertions intact; this task is a schema/compile migration and should not change runtime validation semantics yet.

Must-haves:
- `SkillDef` metadata is required in Rust, not optional/defaulted away.
- Stable reason codes are machine identifiers, not display strings.
- No UI/CLI/windowed skill-ID-specific legality table is introduced.
- Fixture migration preserves current test intent for damage, revive, SP, toughness, status, follow-up, and ultimate tests.

Failure Modes:
- **Malformed fixture metadata**: compile may pass but later validation may fail; use clear helper names and comments for revive vs offensive vs Row-deferred fixtures.
- **Serde compatibility drift**: unknown fields must fail after `deny_unknown_fields`; do not add broad catch-all variants.
- **Mechanical churn misses**: `cargo check --tests` is the guard for all direct `SkillDef` literals.

Load Profile:
- Shared resources: none beyond compile/test time.
- Per-operation cost: serde structs are static data; no runtime allocation beyond loading existing skill data.
- 10x breakpoint: a larger catalog increases validation iteration linearly; no complex lookup is introduced in this task.

Negative Tests:
- Planned in T02 after canonical RON is migrated; T01 should not claim validation coverage beyond type-level required fields.

Verification:
- `cargo check --tests`
- `rg "SkillDef \\{" src tests` shows all remaining literals explicitly include `targeting:` and `implementation:` or are inside helper constructors that do.

Inputs:
- `src/data/skills_ron.rs` — existing SkillDef, Effect, TargetShape, and serde tests.
- `src/combat/resolution_tests.rs` — direct SkillDef fixtures for resolution behavior.
- `src/combat/follow_up_tests.rs` — direct SkillDef fixtures for follow-up behavior.
- `src/combat/turn_system/tests.rs` — direct SkillDef fixtures for turn-system behavior.
- `tests/resource_caps.rs` — direct SkillDef fixtures for resource tests.
- `tests/sp_economy.rs` — direct SkillDef fixtures for SP tests.
- `tests/revive_semantics.rs` — revive fixture semantics.
- `tests/patamon_revive.rs` — revive fixture semantics.
- `tests/target_shape_truthfulness.rs` — Row/AllEnemies shape fixtures from S02.
- `tests/toughness_enemy_only.rs` — toughness fixtures.
- `tests/status_effect_apply.rs` — status fixtures.
- `tests/status_effect_integration.rs` — status fixtures.
- `tests/boundary_contract.rs` — boundary contract fixtures.
- `tests/combat_coherence.rs` — coherence fixtures.
- `tests/damage_breakdown_log.rs` — damage fixture.
- `tests/encounter_e2e.rs` — encounter fixture.
- `tests/event_stream.rs` — event stream fixture.
- `tests/status_accuracy.rs` — status accuracy fixture.
- `tests/toughness_categories.rs` — toughness category fixture.
- `tests/ultimate_meter.rs` — ultimate fixture.

Expected Output:
- `src/data/skills_ron.rs` — new metadata schema attached to SkillDef.
- `src/combat/resolution_tests.rs` — fixtures migrated to explicit metadata.
- `src/combat/follow_up_tests.rs` — fixtures migrated to explicit metadata.
- `src/combat/turn_system/tests.rs` — fixtures migrated to explicit metadata.
- `tests/resource_caps.rs` — fixtures migrated to explicit metadata.
- `tests/sp_economy.rs` — fixtures migrated to explicit metadata.
- `tests/revive_semantics.rs` — fixtures migrated to explicit metadata.
- `tests/patamon_revive.rs` — fixtures migrated to explicit metadata.
- `tests/target_shape_truthfulness.rs` — fixtures migrated to explicit metadata.
- `tests/toughness_enemy_only.rs` — fixtures migrated to explicit metadata.
- `tests/status_effect_apply.rs` — fixtures migrated to explicit metadata.
- `tests/status_effect_integration.rs` — fixtures migrated to explicit metadata.
- `tests/boundary_contract.rs` — fixtures migrated to explicit metadata.
- `tests/combat_coherence.rs` — fixtures migrated to explicit metadata.
- `tests/damage_breakdown_log.rs` — fixtures migrated to explicit metadata.
- `tests/encounter_e2e.rs` — fixtures migrated to explicit metadata.
- `tests/event_stream.rs` — fixtures migrated to explicit metadata.
- `tests/status_accuracy.rs` — fixtures migrated to explicit metadata.
- `tests/toughness_categories.rs` — fixtures migrated to explicit metadata.
- `tests/ultimate_meter.rs` — fixtures migrated to explicit metadata.

## Inputs

- `src/data/skills_ron.rs`
- `src/combat/resolution_tests.rs`
- `src/combat/follow_up_tests.rs`
- `src/combat/turn_system/tests.rs`
- `tests/resource_caps.rs`
- `tests/sp_economy.rs`
- `tests/revive_semantics.rs`
- `tests/patamon_revive.rs`
- `tests/target_shape_truthfulness.rs`
- `tests/toughness_enemy_only.rs`
- `tests/status_effect_apply.rs`
- `tests/status_effect_integration.rs`
- `tests/boundary_contract.rs`
- `tests/combat_coherence.rs`
- `tests/damage_breakdown_log.rs`
- `tests/encounter_e2e.rs`
- `tests/event_stream.rs`
- `tests/status_accuracy.rs`
- `tests/toughness_categories.rs`
- `tests/ultimate_meter.rs`

## Expected Output

- `src/data/skills_ron.rs`
- `src/combat/resolution_tests.rs`
- `src/combat/follow_up_tests.rs`
- `src/combat/turn_system/tests.rs`
- `tests/resource_caps.rs`
- `tests/sp_economy.rs`
- `tests/revive_semantics.rs`
- `tests/patamon_revive.rs`
- `tests/target_shape_truthfulness.rs`
- `tests/toughness_enemy_only.rs`
- `tests/status_effect_apply.rs`
- `tests/status_effect_integration.rs`
- `tests/boundary_contract.rs`
- `tests/combat_coherence.rs`
- `tests/damage_breakdown_log.rs`
- `tests/encounter_e2e.rs`
- `tests/event_stream.rs`
- `tests/status_accuracy.rs`
- `tests/toughness_categories.rs`
- `tests/ultimate_meter.rs`

## Verification

cargo check --tests && rg "SkillDef \\{" src tests

## Observability Impact

Validation-related types and errors should be debuggable by `Debug` output and tests, but this task does not add runtime logging or event surfaces.
