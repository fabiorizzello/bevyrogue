# S04 Research — Pure legality and affordance query API

## Scope and requirement focus

S04 owns the first executable portion of R084: a pure, DSL-backed query API that tests can call to get action and target affordances before execution. It supports R085 by making implementation/deferred/hidden state queryable for skills and toughness, but S05/S06/S07 will wire resources, engine enforcement, and UI/CLI consumers more deeply.

Relevant active requirements:
- **R084** — primary. Query must answer action legality and target validity from skill DSL + immutable combat snapshot, with stable reason codes and no per-skill hardcoding.
- **R085** — supporting. Query must expose implementation status/deferred/hidden truth for target shapes and current UI-affecting mechanics, especially enemy-only toughness and unsupported shapes/effects.

Useful prior memory:
- MEM076/MEM073: `SkillDef.targeting` is canonical for shape/side/life/self; `SkillDef.implementation` owns implemented/deferred/hidden with stable reason codes.
- MEM071/MEM069: unsupported shapes are currently rejected as `UnimplementedTargetShape:<Shape>` before lifecycle mutation.
- MEM070: toughness exposure is gated through `exposes_toughness_affordance` / `visible_toughness` / `can_apply_toughness_damage` rather than by removing ally `Toughness` structurally.
- MEM066: hard boundary is no skill-ID-specific legality rules in CLI/windowed/UI.

Skill guidance that matters:
- The installed **tdd** skill is directly applicable: build this as a red/green vertical slice around observable query results (revive, offensive, unsupported shape, resource gates) rather than horizontal plumbing.
- The installed **verify-before-complete** skill applies to downstream executors: before claiming S04 complete, produce fresh `cargo test-dev ...` evidence in the current message/artifact.

## Recommendation

Create a new headless-only combat module, likely `src/combat/action_query.rs`, that defines:

1. Plain snapshot structs independent of Bevy ECS borrows.
2. Status/reason enums mirroring `docs/skill_legality_contract.md`.
3. Pure functions to evaluate one action and all candidate targets from `SkillBook`, `UnitSkills`, resources, phase, active unit, and unit snapshots.
4. A small Bevy-world adapter only if needed for tests/consumers; keep the core evaluator purely data-in/data-out.

Do **not** wire this into the engine in S04 except possibly sharing helper code. S06 is the roadmap point for authoritative engine integration. S04 should prove the pure API contract with deterministic tests and leave integration seams clean.

## Implementation landscape

### Existing DSL source of truth

`src/data/skills_ron.rs` now contains the relevant DSL metadata:
- `TargetShape::{Single, Row, AllEnemies, SelfOnly}`
- `TargetSide::{Ally, Enemy, Any}`
- `TargetLife::{Alive, Ko, Any}`
- `SelfTargetRule::{Forbid, Allow}`
- `SkillTargeting { shape, side, life, self_rule }`
- `SkillImplementation::{Implemented, Deferred { reason }, Hidden { reason }}`
- `SkillDef { id, sp_cost, targeting, implementation, effects, ... }`

Current `LegalityReasonCode` is incomplete versus the doc. It has:
`UnimplementedTargetShape`, `UnimplementedEffect`, `WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetNotDamaged`, `TargetIsSelf`, `TargetIsCommander`, `NoValidTargets`, `ToughnessEnemyOnly`.

The contract doc/tests require additional action/resource/mechanic codes:
`NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, `SpShortfall`, `UltimateNotReady`, `TargetNotFound`, `TamerGaugeDeferred`, `TamerCommandDeferred`, `ChargedTelegraphDeferred`, `EnemyTraitDeferred`, `EnergyCapReached`.

Adding enum variants should not break RON because canonical data only references existing variants. It will make tests and future query output use the same enum instead of stringly-typed reasons.

### Existing engine behavior to mirror later

`src/combat/resolution.rs`:
- `resolve_action(intent, kit, book)` maps an `ActionIntent` to `ResolvedAction` using the actor kit and skill book.
- `ResolvedAction.target_shape` is copied from `skill.targeting.shape`.
- `target_shape_rejection_reason(shape)` returns `Some("UnimplementedTargetShape:{shape:?}")` for non-`Single` shapes.
- `apply_effects` still contains late validation using display strings: commander, attacker KO, revive target not KO, attack target KO, SP shortfall, ultimate readiness.

`src/combat/turn_system/pipeline.rs`:
- `step_declaration` resolves skill and rejects unsupported shapes before lifecycle events.
- `step_app` still performs actor/target KO/stun validation directly and emits display-string `OnActionFailed` events.
- There are old commented snapshot blocks in `step_app`; do not reuse them directly, but they show the intended seam: collect immutable actor/target snapshots before mutation.

S04 should not attempt to reconcile all these display strings yet; S06 will require engine/query parity. But S04 should produce reason codes that S06 can map to failure strings/events.

### Existing snapshot-like code

`src/combat/observability.rs` has `capture_validation_snapshot(world)` that builds a Bevy-world snapshot for diagnostics. It is useful precedent but not enough for S04:
- It requires resources (`CombatState`, `SpPool`, `ActionLog`) and some components.
- It hides ally toughness using `visible_toughness`, which is correct for UI diagnostics.
- It does not include `UnitSkills`, commander markers, active unit, raw stun state as a boolean, or enough resource detail for action legality.

Recommendation: create new query snapshot types instead of coupling legality to `ValidationSnapshot`. Reuse `visible_toughness`/`exposes_toughness_affordance` helpers where relevant.

### Toughness helpers already exist

`src/combat/toughness.rs` provides:
- `exposes_toughness_affordance(team, Option<&Toughness>) -> bool`
- `visible_toughness(team, Option<&Toughness>) -> Option<ToughnessView>`
- `can_apply_toughness_damage(team, Option<&Toughness>) -> bool`

S04 can expose a query-side toughness affordance using these helpers. For allies or zero/maxless bars, return hidden/disabled with `ToughnessEnemyOnly` rather than showing a break bar.

### Consumers currently hardcode legality and should be left for S07

`src/bin/combat_cli.rs` currently:
- Offers Basic, all skill IDs, and Ultimate only when ready.
- Builds targets as all non-KO non-self units on both teams, which hides KO allies and allows wrong-side targets.

`src/ui/combat_panel.rs` currently:
- Uses `order.future_preview.first()` for active ally, not necessarily the same active-unit source as engine.
- Enables action buttons using local conditions.
- Allows clicking only non-KO enemies as targets.

These are S07 consumers. S04 should expose data shapes that make replacing these paths straightforward, but should not couple the pure query to egui or inquire.

### Enemy AI currently has a parallel target precondition

`src/combat/enemy_ai.rs` is pure already but assumes callers filter targets to living non-commander allies. Eventually it should use the query, but this is not S04 unless tests want to prove pure API suitability for AI.

## Canonical skill-data observations

A small metadata summary of `assets/data/skills.ron` showed:
- 72 skills total with `targeting` and `implementation` metadata.
- `Implemented`: 60, `Deferred`: 6, `Hidden`: 6.
- Shapes: `Single`: 61, `Row`: 6, `SelfOnly`: 5, `AllEnemies`: 0.
- Ally-targeting skills: 2; KO-targeting skills: 2.
- Deferred/non-single examples include `heat_viper`, `greymon_ult`, `mega_blaster_aoe`, `kabuterimon_ult`, `kyubimon_ult`, `angemon_ult`.
- Hidden self-only form identity examples include `greymon_form_identity`, `garurumon_form_identity`, `kabuterimon_form_identity`, `kyubimon_form_identity`.

Use canonical examples for query tests where possible, but inline fixtures are safer for narrow behavior such as heal-like/damaged-ally semantics because no first-class heal effect exists yet.

## Suggested API shape

Names are suggestions, not requirements:

```rust
// src/combat/action_query.rs
pub struct CombatQuerySnapshot {
    pub phase: CombatPhase,
    pub active_unit: Option<UnitId>,
    pub sp_current: i32,
    pub sp_max: i32,
    pub units: Vec<UnitQuerySnapshot>,
}

pub struct UnitQuerySnapshot {
    pub id: UnitId,
    pub team: Team,
    pub hp_current: i32,
    pub hp_max: i32,
    pub ko: bool,
    pub stunned: bool,
    pub commander: bool,
    pub skills: Option<UnitSkills>,
    pub ultimate_current: i32,
    pub ultimate_trigger: i32,
    pub toughness: Option<ToughnessView>,
}

pub enum ActionQueryKind {
    Basic,
    Skill(SkillId),
    Ultimate,
}

pub struct ActionAffordance {
    pub action: ActionQueryKind,
    pub skill_id: Option<SkillId>,
    pub status: ActionStatus,
    pub implementation: ImplementationStatus,
    pub resource: Option<ResourceStatus>,
    pub targets: Vec<TargetAffordance>,
}

pub struct TargetAffordance {
    pub target: UnitId,
    pub status: TargetStatus,
}
```

Status enums should match doc semantics:
- `ActionStatus::{Enabled, Disabled { reason }, Deferred { reason }, Hidden { reason }}`
- `TargetStatus::{Legal, Illegal { reason }, Deferred { reason }, Hidden { reason }}`
- `ResourceStatus::{Ready, Insufficient { reason, current, required }, Deferred { reason }, Hidden { reason }}`
- `ImplementationStatus::{Implemented, Deferred { reason }, Hidden { reason }}`

Use `LegalityReasonCode` as the reason type, after extending it with the missing doc reason codes. If display text is needed, add a method like `as_code()` or `display_code()`; do not use localized text in tests.

## Core legality rules to implement in S04

Action-level checks, in stable priority order:
1. Missing actor or missing kit/skill should return `MissingSkill` or no affordance depending on API call shape. Prefer explicit `MissingSkill` for requested actions.
2. Non-active actor: `Disabled(NotActiveUnit)`.
3. Wrong phase: `Disabled(WrongPhase)` unless the phase is `WaitingAction`.
4. Actor KO: `Disabled(AttackerKo)`.
5. Actor stunned: `Disabled(AttackerStunned)`.
6. Missing skill ID from kit or skill book: `Disabled(MissingSkill)`.
7. `SkillImplementation::Deferred/Hidden`: map to `ActionStatus::Deferred/Hidden` and `ImplementationStatus` with the same reason.
8. Unsupported `TargetShape` when marked implemented or when evaluating targets: `Deferred(UnimplementedTargetShape)` or target-level `Deferred(UnimplementedTargetShape)`; match the contract and S02/S03 behavior that non-`Single` is not executable.
9. SP shortfall for skills: `ResourceStatus::Insufficient { reason: SpShortfall, current, required }` and `ActionStatus::Disabled(SpShortfall)`.
10. Ultimate not ready: `ResourceStatus::Insufficient { reason: UltimateNotReady, current, required }` and `ActionStatus::Disabled(UltimateNotReady)`.
11. If all action/resource/implementation checks pass but no target is legal: `ActionStatus::Disabled(NoValidTargets)`.
12. Otherwise `ActionStatus::Enabled`.

Target-level checks:
1. Missing target on direct target query: `TargetNotFound`.
2. Commander target: `TargetIsCommander`.
3. Self targeting: if target is actor and `SelfTargetRule::Forbid`, `TargetIsSelf`; if a future `SelfOnly` shape is implemented, enforce self required separately (not expressible today except shape).
4. Side mismatch from `SkillTargeting.side`: `WrongSide`. For `TargetSide::Ally`, same team as actor; for `Enemy`, different team; for `Any`, allow either.
5. Life mismatch from `SkillTargeting.life`: `TargetKo` when target is KO but Alive required; `TargetNotKo` when target is not KO but KO required.
6. Heal-like/damaged target fixtures: there is no `Heal` effect yet. If S04 needs to prove `TargetFullHp`/`TargetNotDamaged`, either add a query-only `requires_missing_hp` derivation from a new DSL bit (larger) or use a fixture skill with an existing effect and a documented targeting extension. Planner should decide whether this belongs in S04 or S09; roadmap explicitly says S04 tests should cover heal-like disabled state, so adding a small `TargetHealth`/`TargetHpRule` field may be the cleanest but touches canonical RON.
7. Unsupported shapes/effects: `Deferred(UnimplementedTargetShape)` or `Deferred(UnimplementedEffect)` rather than illegal.

Important ambiguity: `SelfTargetRule` currently has only `Forbid` and `Allow`; it cannot express “self required.” `TargetShape::SelfOnly` exists, but all canonical self-only items are hidden form-identity declarations. If S04 must expose future self-targeted actions truthfully, consider adding `SelfTargetRule::Require` or treating `TargetShape::SelfOnly` as `Hidden/Deferred` until S09. Do not silently make `SelfOnly` equivalent to `Single`.

## Natural seams for planner decomposition

1. **Type vocabulary and reason-code expansion**
   - Files: `src/data/skills_ron.rs`, new `src/combat/action_query.rs`, `src/combat/mod.rs`.
   - Add missing `LegalityReasonCode` variants and status structs/enums.
   - Keep serde derives if these outputs may later feed UI/CLI; not required but useful.

2. **Pure snapshot/action evaluator**
   - Files: new `src/combat/action_query.rs`; tests in `tests/action_affordance_query.rs` or similar functional name.
   - Implement skill lookup from `ActionQueryKind` and actor `UnitSkills`.
   - Implement phase/active/KO/stun/SP/ultimate/implementation checks.

3. **Target evaluator**
   - Same module/test file.
   - Implement side/life/self/commander/shape rules.
   - Prove offensive live enemy, revive KO ally, wrong side, live/KO mismatches.

4. **Toughness and implementation affordance helpers**
   - Files: new action query module plus `src/combat/toughness.rs` usage.
   - Expose enemy toughness as implemented/visible, ally toughness as hidden/disabled with `ToughnessEnemyOnly`.
   - Expose `SkillImplementation::Deferred/Hidden` in query output.

5. **Optional Bevy adapter**
   - Files: action query module or separate adapter module.
   - Only add if tests or next slices need it now. A function that collects `CombatQuerySnapshot` from a `World` will help S06/S07, but the core evaluator must remain testable without Bevy mutation.

## Riskiest/most ambiguous points

- **Heal-like semantics are under-modeled.** `TargetFullHp`/`TargetNotDamaged` exist in docs and enum, but there is no `Heal` effect or targeting metadata to say “requires damaged target.” If S04 acceptance truly requires a heal-like fixture, planner should choose between:
  - Add a small DSL metadata field such as `TargetHpRule::{Any, Damaged, Full?}` and migrate all RON with `Any`; or
  - Defer heal-like executable behavior and only test reason-code plumbing with an artificial query-only fixture. The first is cleaner for future UI but broadens RON migration.
- **Action status priority needs to be fixed by tests.** Example: a KO actor using a hidden skill with no targets could reasonably return `Hidden`, `AttackerKo`, or `NoValidTargets`. Pick and document priority to avoid brittle downstream parity.
- **Effective SP cost must include Child discount eventually.** Current `apply_effects` computes the Child basic-streak discount late from `BasicStreak`. If S04 includes SP readiness, the snapshot must optionally include `BasicStreak`/evo stage or this must be explicitly deferred to S05/S06. `UnitQuerySnapshot` should probably include `evo_stage` and `basic_streak_count` if SP parity is in scope.
- **Ultimate readiness needs actor `UltimateCharge`.** Snapshot should include current/trigger; actions should return `UltimateNotReady` with current/required.
- **`LegalityReasonCode` currently derives no `Copy`.** Many statuses will clone reasons. That is okay, but variants are fieldless and can be made `Copy` if desired. Be careful because it is serialized in RON.
- **Do not use `ValidationSnapshot` directly.** It intentionally hides details and omits skills/commander/active-unit resource state needed for legality.

## Verification plan

Recommended focused tests:

- `cargo test-dev --test action_affordance_query`
  - offensive implemented single-target skill: live enemy legal; ally `WrongSide`; KO enemy `TargetKo`; commander `TargetIsCommander`; self `TargetIsSelf` when forbidden.
  - revive implemented skill: KO ally legal; live ally `TargetNotKo`; enemy `WrongSide`.
  - action-level disabled: `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`.
  - SP shortfall: `ResourceStatus::Insufficient { reason: SpShortfall, current, required }` and disabled action.
  - ultimate not ready: `UltimateNotReady`.
  - deferred Row skill: action/targets not enabled and reason `UnimplementedTargetShape`.
  - hidden self-only form-identity-like skill: status `Hidden(UnimplementedEffect)`.
  - no legal targets: `NoValidTargets`.
  - toughness: enemy visible/implemented; ally hidden/disabled `ToughnessEnemyOnly`.

Regression/focused existing suites:
- `cargo test-dev skills_ron`
- `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs`

Before completing S04, run:
- `cargo test-dev`

Windowed check is not required by S04 roadmap wording, but running `cargo check --features "dev windowed"` is safe if any public type changes affect UI imports.

## Skill discovery

Core technologies are Rust, Bevy ECS, serde/RON. Installed skills do not include a Rust/Bevy-specific implementation skill. I checked the skills registry but did not install anything.

Promising optional skills:
- `npx skills add bfollington/terma@bevy` — 117 installs; Bevy-specific.
- `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert` — 108 installs; likely relevant for ECS patterns.
- `npx skills add mindrally/skills@rust` — 243 installs; general Rust.
- `npx skills add existential-birds/beagle@serde-code-review` — 18 installs; potentially useful if expanding serde/RON schema.
- `npx skills add udapy/rust-agentic-skills@ron-specialist` — 13 installs; RON-specific but lower adoption.

No external library docs were needed; this slice uses established in-repo Rust/Bevy patterns.

## Suggested planner task order

1. Add `action_query` module with status/reason types and minimal snapshot structs; extend `LegalityReasonCode` with contract-required variants.
2. Write failing pure tests for offensive/revive/deferred target affordances.
3. Implement target evaluation.
4. Add action/resource tests for active phase, active unit, KO/stun, SP, ultimate readiness, missing skill, no legal targets.
5. Implement action evaluator and ensure output includes target list.
6. Add toughness/implementation visibility tests and helpers.
7. Run focused tests and full `cargo test-dev`.
