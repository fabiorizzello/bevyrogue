---
verdict: pass
remediation_round: 0
---

# Milestone Validation: M012

## Success Criteria Checklist
- [x] **R084 is validated: action legality and target validity are data-driven, queryable, and shared by engine/UI/CLI.** | S01 locked the vocabulary and 24+ reason codes; S03 made SkillDef carry explicit targeting/implementation metadata in DSL; S04 delivered the pure `query_action_affordance()` API consuming DSL snapshots; S06 wired the same pure query into `resolve_action_system()` so the engine rejects illegal intents through the same surface; S07 confirmed CLI and windowed adapters consume the shared query with no local legality logic.

- [x] **R085 is validated: UI-affecting mechanics are either implemented truthfully or represented as queryable deferred/hidden affordances.** | S02 fixed enemy-only toughness and TargetShape truthfulness; S03 made deferred/hidden explicit in DSL data; S08 added typed enemy counterplay declarations; S09 reclassified all ToFixNow gap matrix entries to Implemented or Deferred with zero ToFixNow rows remaining — verified by doc-contract tests.

- [x] **Existing combat behavior remains green under `cargo test-dev`.** | S06 reports full suite passed (131 lib + 132 main tests + all integration tests, 0 failures). S07 confirms full suite pass. S08 confirms full suite pass, 0 failures. S09 confirms full suite pass, all binaries, 0 failures.

- [x] **Windowed path compiles after UI-affordance integration (`cargo check --features "dev windowed"`).** | S02, S03, S05, S06, S07, S08, S09 all report `cargo check --features "dev windowed"` passed. S07 notes windowed query wiring specifically done for the consumer integration.

- [x] **Revive/Heal-like/Offensive examples demonstrate legal target filtering before execution and authoritative engine rejection after execution attempt.** | S04 delivered 18-test suite covering revive (KO-ally targeting), offensive (live enemies), heal-like (damaged-target), wrong-side, and no-valid-target cases via pure query. S06 delivered `engine_legality_integration` (7/7) with forced illegal intents producing `OnActionFailed` before lifecycle events. S07 confirmed revive-like KO-ally targeting flows through query output in CLI/windowed consumers.

- [x] **Enemy-only Toughness and TargetShape semantics no longer produce false UI claims.** | S02 made toughness team-aware (ally toughness hidden, zero-max hidden, positive enemy toughness visible) and added `UnimplementedTargetShape:<Shape>` rejection for Row/AllEnemies before mutation. S03 preserved this via DSL-driven target-shape propagation. S04/S07 expose toughness visibility through the pure query surface.

- [x] **Energy gain caps are enforced in the actual pipeline.** | S05 wired live `GrantEnergy` through the round-based tracker and actual `Energy.max` clamping; `resource_caps` test suite (6/6) covers same-round cap enforcement, truthful `EnergyGained` emission, max-clipping, tracker reset, Child boost, and Form Identity energy under caps. Form Identity (10/10) stayed green.

- [x] **No UI or CLI code contains per-skill legality hardcoding.** | S07 introduced static source-scan tests in `tests/action_affordance_consumers.rs` that grep CLI (`src/bin/combat_cli.rs`) and windowed panel (`src/ui/combat_panel.rs`) for hardcoded KO/team/skill-ID legality branches. S08 extended these scans to cover enemy counterplay consumer blocks. S07 reports 7/7 consumer tests and 23/23 affordance-query tests passing.

## Slice Delivery Audit
All 9 slices have SUMMARY.md files with passing verification results:

- **S01** ✅ — Produced gap matrix (`docs/combat_ui_readiness_gap_matrix.md`) and legality contract (`docs/skill_legality_contract.md`) with 17 doc-contract tests passing.
- **S02** ✅ — Enemy-only toughness and TargetShape truthfulness fixed. 18 tests passing across 6 suites. Windowed compile green.
- **S03** ✅ — All 72 canonical skills migrated to explicit `targeting`/`implementation` DSL metadata. `skills_ron`, revive, target-shape, doc-contract tests passing. Windowed compile green.
- **S04** ✅ — Pure `query_action_affordance()` API delivered in `src/combat/action_query.rs`. 18/18 affordance-query tests. Windowed compile green.
- **S05** ✅ — Energy caps wired in live pipeline. `resource_caps` (6/6), `form_identity` (10/10), `skills_ron`, windowed compile all green.
- **S06** ✅ — Engine validation uses preflight query; `engine_legality_integration` (7/7). Full test suite (263+ tests) passing. Windowed compile green.
- **S07** ✅ — CLI and windowed consumers use `query_action_affordance()`; consumer source-scan tests (7/7) + affordance-query (23/23). Full suite passing. Windowed compile green.
- **S08** ✅ — Enemy counterplay/charged-attack typed declarations added; `query_enemy_trait_affordances()` delivered. Counterplay (3/3), consumer scans (13/13), roster (2/2). Full suite passing. Windowed compile green.
- **S09** ✅ — Gap matrix reclassified (zero ToFixNow rows remaining); doc-contract tests (7+10) green. UI handoff doc produced. Full suite passing.

No outstanding follow-ups or known limitations in any slice. One pre-existing `form_identity` regression noted in S04 was resolved by S05.

## Cross-Slice Integration
All tracked cross-slice boundaries are honored — every producer SUMMARY confirms delivery, every consumer SUMMARY confirms explicit consumption:

| Integration Point | Status |
|---|---|
| S01 → S03: Contract vocabulary used by DSL naming | PASS |
| S01 → S04: Contract vocabulary consumed by query API return types | PASS |
| S01 → S06: Engine parity requirement (OnActionFailed derives from same reason codes) | PASS |
| S01 → S07/S08: No-skill-ID-specific-UI-rule hard boundary enforced by source-scan tests | PASS |
| S02 → S04: Toughness helper functions consumed by query API | PASS |
| S02 → S06: Stable `UnimplementedTargetShape` rejection and `OnActionFailed` pattern | PASS |
| S03 → S04: `SkillDef.targeting`/`implementation` consumed by query API (explicitly named) | PASS |
| S04 → S06: Preflight query consumed by engine validation; parity proven by `engine_legality_integration` | PASS |
| S04 → S07: Preflight query consumed by CLI/windowed affordance integration | PASS |
| S05 → S07: Energy cap affordance data integrated into CLI snapshot inputs | PASS |
| S05 → S04 vocabulary alignment: Resource reason codes kept aligned across slices | PASS |
| S06 → S07: `build_snapshot_from_ecs()` pattern reused by CLI/windowed | PASS |
| S04/S06/S07 → S08: Query vocabulary reused for counterplay declarations | PASS |
| S02–S08 → S09: Gap matrix reclassification reflects actual delivery; zero ToFixNow remaining | PASS |

No integration gap was found. The one pre-existing `form_identity` regression noted in S04 was resolved within the milestone by S05 and is not a cross-slice boundary failure.

## Requirement Coverage
| Requirement | Status | Evidence |
|---|---|---|
| **R084** — Skill legality and targeting data-driven, queryable, shared by engine/UI/CLI | **COVERED** | S01 defined the status vocabulary and 24+ reason codes with 10 doc-contract tests enforcing coverage. S03 migrated all 72 canonical skills to explicit `targeting`/`implementation` DSL metadata. S04 introduced `src/combat/action_query.rs` with a pure query surface (18/18 tests). S06 wired engine rejection through the same legality surface with integration parity tests (7/7). S07 confirmed CLI and windowed adapters consume the shared query without re-encoding legality rules (7/7 consumer tests + static source-scan guards). Full suite: 0 failures. |
| **R085** — UI-affecting mechanics implemented truthfully or declared as queryable deferred/hidden affordances | **COVERED** | S01 produced the executable gap matrix classifying 18+ mechanics. S02 made ally Toughness hidden, enemy-only Toughness truthful, and Row/AllEnemies TargetShape explicitly rejected with `UnimplementedTargetShape`. S05 wired live Energy-cap enforcement in pipeline; Tamer/Child resource affordances queryable with stable reason codes. S08 added typed `EnemyCounterplayKind`/`ChargedAttackDeclaration` declarations with hardcoding-scan guards on CLI/windowed consumers. S09 reclassified all ToFixNow entries — zero remaining; all in-scope mechanics are either `Implemented` or `Deferred` with explicit query metadata. All doc-contract tests green (7+10). |

Both requirements are fully covered across S01–S09 with executable test evidence, no hardcoded fallbacks in consumer adapters, and zero unresolved `ToFixNow` gap matrix entries.

## Verification Class Compliance
| Class | Planned Check | Evidence | Verdict |
|---|---|---|---|
| Contract | Every implementation slice runs targeted legality tests; S07+ also runs `cargo check --features "dev windowed"` | S01: 17 doc-contract tests. S02: targeted suite + windowed check. S03: skills_ron + revive/target-shape suites + windowed check. S04: 18-test affordance suite + windowed check. S05: resource_caps (6/6) + form_identity (10/10) + windowed check. S06: engine_legality_integration (7/7) + full suite. S07: consumer (7/7) + affordance-query (23/23) + full suite + windowed check. S08: counterplay (3/3) + consumers (13/13) + roster (2/2) + full suite + windowed check. S09: doc-contract tests (7+10) + full suite. | PASS |
| Integration | Legal and illegal `ActionIntent`s injected through Bevy message bus; emitted failure reasons compared with preflight; TargetShape, Toughness, Energy cap verified through real pipeline paths | S06: `engine_legality_integration` suite forces illegal intents into Bevy bus, verifies same reason code, exactly one failure event, no lifecycle events, unchanged state. S05: Energy cap enforced in live `GrantEnergy` resolution verified by `resource_caps`. S02: TargetShape rejection before mutation verified by `target_shape_truthfulness`. Parity between preflight query and runtime rejection proven by S06 design (`query_intent_legality()` same priority order as preflight). | PASS |
| Operational | `CombatEventKind::OnActionFailed` carries machine-readable reason; Energy gain and deferred-affordance states inspectable without UI-only logic | S06: `OnActionFailed` carries stable Debug-form `LegalityReasonCode` strings (e.g., `TargetNotKo`, `WrongSide`, `AttackerStunned`). ActionLog mirrors canonical reason codes. S04/S07: deferred affordances surface `ImplementationStatus::Deferred/Hidden` via pure query, no UI-only logic. S05: `EnergyGained` events emit truthful capped amounts. S08: enemy counterplay deferred states return `ImplementationStatus` variants via `query_enemy_trait_affordances()`. | PASS |
| UAT | CLI/windowed sanity: Revive shows KO ally targets, damage shows live enemies, disabled/deferred actions explain why, no unavailable Tamer/trait/shape affordances shown as usable | S09-UAT.md documents 12 test cases covering gap matrix reclassification, doc-contract tests, data alignment, and UI handoff doc coverage. S07 verified revive KO-ally targeting flows from query output in consumers. S08 verified deferred counterplay traits display with `ImplementationStatus` from query, not hardcoded. No windowed graphical UAT — correctly deferred to next UI milestone as planned. | PASS (within planned scope; graphical UAT explicitly deferred) |


## Verdict Rationale
All three independent reviewers returned PASS. All 8 success criteria are satisfied with concrete test evidence across S01–S09. All 14 tracked cross-slice integration boundaries are honored with explicit producer/consumer confirmation. Both R084 and R085 are fully covered with zero ToFixNow gap matrix entries remaining. All four verification classes (Contract, Integration, Operational, UAT) have passing evidence. The one planned deferral (full graphical windowed UAT) is pre-declared in the milestone plan and is not a gap.
