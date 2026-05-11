---
id: M012
title: "Data-driven skill legality and UI-readiness query surface"
status: complete
completed_at: 2026-05-01T17:22:52.586Z
key_decisions:
  - D053: Extend SkillDef DSL with targeting/legality contract and expose a pure query API shared by engine, CLI, UI, tests — no detached hardcoded UI layer
  - D054: M012 fixes UI-blocking semantics directly (Toughness, TargetShape, Energy caps) and adds declarative queryable placeholders for larger future systems (Tamer Commands, enemy counterplay) rather than fully implementing them
  - D055: Team-aware Toughness helpers gate enemy-only break affordances without structurally removing Toughness from allies, preserving existing damage affinity behavior
  - D056: TargetHpRule field in SkillTargeting allows revive/heal-like target constraints to live in DSL data rather than UI branches
  - D057: Separate SP snapshot paths — engine bypass keeps existing SP shortfall lifecycle; explicit-SP path makes CLI/windowed affordances truthful before execution
  - D058: Typed EnemyCounterplayKind enum + ECS component for enemy counterplay declarations, never inferred from ToughnessCategory — ReactiveArmor invariant enforced by dedicated test
key_files:
  - src/combat/action_query.rs
  - src/data/skills_ron.rs
  - assets/data/skills.ron
  - assets/data/units.ron
  - src/data/units_ron.rs
  - src/combat/toughness.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/mod.rs
  - src/bin/combat_cli.rs
  - src/ui/combat_panel.rs
  - tests/action_affordance_query.rs
  - tests/action_affordance_consumers.rs
  - tests/engine_legality_integration.rs
  - tests/toughness_enemy_only.rs
  - tests/target_shape_truthfulness.rs
  - tests/resource_caps.rs
  - tests/enemy_counterplay_affordance.rs
  - tests/skill_legality_contract_docs.rs
  - tests/ui_readiness_gap_matrix_docs.rs
  - docs/skill_legality_contract.md
  - docs/combat_ui_readiness_gap_matrix.md
  - docs/ui_handoff_m012.md
lessons_learned:
  - Write the vocabulary contract (status enums, reason codes, doc artifacts) in S01 before any implementation slice begins — it prevents vocabulary drift across 8 implementation slices and makes doc-contract tests a reliable regression net
  - One legality source of truth must feed both preflight queries and runtime rejection; when S06 wired engine validation to the S04 query API, it eliminated an entire class of engine/UI divergence by construction
  - Source-scan regression tests in consumer files are low-cost high-value guards: they prevent hardcoded skill-ID or team legality paths from creeping back into CLI or windowed adapters without requiring behavioral tests for every possible violation
  - Team-aware helpers (is_enemy_toughness_visible, apply_toughness_damage_if_enemy) satisfy UI-truth requirements without structurally removing components that carry orthogonal data — a useful seam for incremental refactoring
  - Expanding ECS actor query tuples (e.g. adding EnemyCounterplayKit) requires updating every independent Query signature across files not covered by the turn_system alias — follow_up.rs and combat_cli.rs were caught by compile errors but required explicit attention
  - Additive annotation sections in design docs preserve existing string-grep contracts under test coverage — safe pattern for doc updates when doc-contract tests assert specific substrings
---

# M012: Data-driven skill legality and UI-readiness query surface

**Built a unified, data-driven legality and affordance query surface so UI, CLI, AI, and engine all ask the same DSL-backed contract whether an action is legal and which targets are valid — no per-skill hardcoding anywhere.**

## What Happened

M012 delivered the foundational infrastructure for a truthful player-facing UI in bevyrogue. The milestone addressed a structural gap: before M012, UI/CLI code had no authoritative source to ask "is this action legal?" and "which targets are valid?" without either duplicating engine rules or hardcoding skill IDs.

**S01** locked the vocabulary before any implementation began. A gap matrix (`docs/combat_ui_readiness_gap_matrix.md`) catalogued every UI-blocking mechanic, and a legality contract (`docs/skill_legality_contract.md`) defined the stable status/reason taxonomy (`ActionStatus`, `TargetStatus`, `ResourceStatus`, `ImplementationStatus`, `LegalityReasonCode`). Seventeen doc-contract tests using `include_str!` enforce compile-time presence of these artifacts.

**S02** fixed the two most immediately UI-misleading semantics: ally units no longer expose enemy Toughness break affordances (team-aware helpers gate visibility/application), and TargetShape variants other than Single are rejected with a stable `UnimplementedTargetShape:<Shape>` reason before any mutation occurs.

**S03** made skill legality explicit in the DSL itself. `SkillDef` was extended with `targeting` and `implementation` metadata fields; all canonical skills in `skills.ron` were migrated; resolution now copies target shape from declared metadata rather than inferring it from effect lists.

**S04** delivered the pure query API: `query_action_affordance()` accepts an immutable world snapshot and returns per-action, per-target, per-resource, and per-toughness affordances with `ActionStatus`/`TargetStatus` enums and stable `LegalityReasonCode` identifiers — no display-string parsing required. The `TargetHpRule::Any`/`RequireDamaged`/`RequireKO` field allows revive/heal-like target constraints to live in data rather than UI branches.

**S05** enforced Energy caps in the live `GrantEnergy` resolution path and added queryable deferred declarations for Tamer Gauge/Commands and Child boost dependency, with stable `Deferred` status and reason codes so UI can hide or explain absent affordances without hardcoding.

**S06** wired engine validation to the same pure query surface: illegal `ActionIntent` payloads emitted to the Bevy bus are rejected before `step_declaration`, mutation, or lifecycle events, using the same `LegalityReasonCode` strings the preflight query would return. A readonly ECS snapshot adapter avoids borrow conflicts with the mutable action pipeline.

**S07** made CLI and windowed affordances consume the shared query: action menus, target menus, and revive-like KO-ally targeting all flow through `query_action_affordance()` rather than local legality heuristics. Source-scan regression tests prevent per-skill or per-team hardcoding from returning in either adapter file.

**S08** added typed enemy counterplay declarations — `EnemyCounterplayKind` enum (`TempoAnchor`, `TypeTrap`, `ReactiveArmor`, `BreakSeal`) and `ChargedAttackDeclaration` — propagated from `UnitDef` RON through a new `EnemyCounterplayKit` ECS component into `UnitQuerySnapshot`. Pure query helpers expose implementation/deferred/hidden status using the existing vocabulary. A critical invariant: `ReactiveArmor` is never inferred from `ToughnessCategory::Armored` — enforced by dedicated test.

**S09** completed the documentation layer: gap matrix reclassified to reflect M012 outcomes (all ToFixNow items resolved), data-alignment notes added to `combat_design.md`, and a compact UI handoff doc (`docs/ui_handoff_m012.md`) written for the next UI-building milestone.

Final verification: `cargo test` passes all suites (0 failures); `cargo check --features "dev windowed"` succeeds; 73 files changed, 8381 insertions vs master.

## Success Criteria Results

- **R084 validated** ✅ — DSL-backed legality query shared by engine (S06), CLI/windowed adapters (S07), and all test fixtures (S04). Revive targets only KO allies, TargetHpRule enforced from data, illegal ActionIntent rejected by engine with stable reason codes.
- **R085 validated** ✅ — Ally Toughness break affordances gated (S02); Row/AllEnemies explicitly rejected as UnimplementedTargetShape (S02); Energy caps enforced in live pipeline (S05); Tamer/Child deferred declarations queryable (S05); enemy counterplay typed and queryable (S08); docs/data agree post-S09.
- **`cargo test` green** ✅ — All test suites pass, 0 failures across full integration suite.
- **Windowed compiles** ✅ — `cargo check --features "dev windowed"` finishes cleanly.
- **Revive/Heal-like/Offensive target filtering** ✅ — `tests/revive_semantics.rs`, `tests/patamon_revive.rs`, `tests/action_affordance_query.rs` prove legal target filtering before execution and engine rejection after illegal intent.
- **Enemy-only Toughness / TargetShape truthfulness** ✅ — `tests/toughness_enemy_only.rs`, `tests/target_shape_truthfulness.rs` confirm false-affordance fixes.
- **Energy caps in pipeline** ✅ — `tests/resource_caps.rs` (6 tests) confirms same-round cap enforcement and truthful EnergyGained emission.
- **No per-skill legality hardcoding** ✅ — Source-scan tests in `tests/action_affordance_consumers.rs` assert no skill-ID or team-hardcoded legality paths in CLI or windowed adapters.

## Definition of Done Results

- **All 9 slices complete** ✅ — S01 through S09 all marked complete with passing verification.
- **All slice SUMMARY.md files exist** ✅ — Confirmed present for S01–S09.
- **Cross-slice integration coherent** ✅ — S06 engine validation uses S04 query API snapshot adapter; S07 CLI/windowed uses same query with explicit-SP path (D057); S08 consumer tests guard the same no-hardcoding contract introduced in S07; S09 doc-contract tests assert all vocabulary introduced in S01 remains present after data-alignment additions.
- **Doc-contract tests pass** ✅ — `tests/skill_legality_contract_docs.rs` and `tests/ui_readiness_gap_matrix_docs.rs` green.

## Requirement Outcomes

- **R084** (active → validated): Action legality and targeting are data-driven via SkillDef DSL metadata (S03), queryable via `query_action_affordance()` (S04), enforced by engine before mutation (S06), and consumed truthfully by CLI/windowed without skill-ID branches (S07).
- **R085** (active → validated): All UI-blocking semantic mismatches resolved: ally Toughness (S02), TargetShape executability (S02), Energy caps (S05), Tamer/Child deferred declarations (S05), enemy counterplay declarations (S08), docs/data alignment (S09).

## Deviations

None.

## Follow-ups

Next UI milestone (M007 or a dedicated UI milestone) can consume `query_action_affordance()` and `query_enemy_trait_affordances()` directly from `src/combat/action_query.rs` — the UI handoff doc at `docs/ui_handoff_m012.md` provides the cold-start reference. Deferred mechanics (Tamer Commands/Gauge, TypeTrap, ReactiveArmor full behavior, charged attacks) have queryable declarations and can be promoted to implemented status independently. SP shortfall migration into the canonical early legality guard (D057) would allow collapsing the two snapshot modes.
