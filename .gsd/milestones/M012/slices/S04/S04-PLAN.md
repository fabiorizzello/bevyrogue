# S04: Pure legality and affordance query API

**Goal:** Expose a headless, pure, DSL-backed action legality and target affordance query API that tests can call before execution to get enabled, disabled, deferred, or hidden states with stable reason codes. The query must consume `SkillDef.targeting` / `SkillDef.implementation` metadata rather than skill IDs, cover action/resource preconditions, target validity, unsupported shapes/effects, damaged-target intent, and toughness visibility, and leave engine/UI wiring to later slices.
**Demo:** After this: tests can call one pure function and get action/target affordances with enabled/disabled/deferred state and reasons.

## Must-Haves

- Must-haves:
- `src/combat/action_query.rs` exists, is exported from `src/combat/mod.rs`, and contains pure data-in/data-out snapshot/query types rather than Bevy system mutation.
- `LegalityReasonCode` covers the contract-required action/resource/mechanic reasons needed by S04 and downstream engine/UI parity.
- Target legality is derived from `SkillDef.targeting` metadata: side, life, self rule, optional damaged-HP requirement, commander exclusion, and unsupported target-shape state.
- Action affordance status is derived from immutable query inputs: phase, active unit, actor KO/stun, skill lookup, implementation state, SP availability, ultimate readiness, and whether any legal target exists.
- Query output exposes implementation/resource/target/toughness affordances as machine-readable statuses with `LegalityReasonCode` reasons; no skill-ID-specific rules are introduced.
- `tests/action_affordance_query.rs` proves offensive, revive, heal-like/damaged-target, deferred row, hidden self-only, resource/action precondition, no-valid-target, and toughness affordance scenarios.
- Threat Surface (Q3):
- Abuse: callers may submit arbitrary actor/action/target IDs or stale snapshots; the query must return disabled/illegal/hidden/deferred statuses rather than panic or silently enable invalid combat commands.
- Data exposure: no secrets or player PII are touched; output is combat affordance data only.
- Input trust: snapshots and skill IDs are untrusted API inputs from future CLI/UI/engine adapters, so missing actor, missing target, missing skill, malformed resource values, and unsupported shapes need stable reasons.
- Requirement Impact (Q4):
- Requirements touched: R084 primary, R085 supporting.
- Re-verify: S03 skill metadata parsing/validation, revive semantics, target-shape truthfulness, toughness enemy-only visibility, and the new pure query contract.
- Decisions revisited: D053 and D055 are consumed, not reversed; the plan keeps legality in `SkillDef` metadata and toughness exposure behind team-aware helpers.

## Proof Level

- This slice proves: Contract-level proof. Real runtime required: no, the core evaluator is pure and verified through integration tests. Human/UAT required: no. The proof is that deterministic tests can call one pure query path and observe the same status/reason vocabulary future engine/UI/CLI consumers will use.

## Integration Closure

Upstream surfaces consumed: `src/data/skills_ron.rs` (`SkillDef`, targeting, implementation, reason codes), `src/combat/kit.rs` (`UnitSkills`), `src/combat/state.rs` (`CombatPhase`), `src/combat/types.rs` (`UnitId`, `SkillId`, `DamageTag`), `src/combat/team.rs`, and `src/combat/toughness.rs` helpers. New wiring introduced in this slice: public `src/combat/action_query.rs` module exported from `src/combat/mod.rs`, plus test coverage in `tests/action_affordance_query.rs`. What remains before end-to-end milestone usability: S05 resource dependency declarations/energy caps, S06 authoritative engine validation, and S07 CLI/windowed consumption.

## Verification

- Runtime signals: none added to the Bevy event pipeline in S04 by design. Inspection surfaces: the pure query output itself is the diagnostic surface, with stable `ActionStatus`, `TargetStatus`, `ResourceStatus`, `ImplementationStatus`, and toughness affordance reason codes inspectable in tests and future adapters. Failure visibility: invalid/missing inputs and unsupported mechanics are represented as statuses/reasons instead of panics or display strings. Redaction constraints: none; combat state only.

## Tasks

- [x] **T01: Define the pure action-query vocabulary and DSL health rule** `est:1h`
  Why: S04 needs a stable machine-readable vocabulary before legality rules can be implemented. This task extends the existing DSL contract without skill-ID tables, adds the pure query module boundary, and creates focused compile-time/fixture coverage that downstream tasks can fill in.

Skills: use `tdd` for the contract-first vertical slice and `verify-before-complete` before marking the task complete.

Do:
1. Extend `LegalityReasonCode` in `src/data/skills_ron.rs` with the missing contract reasons: `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, `SpShortfall`, `UltimateNotReady`, `TargetNotFound`, `TamerGaugeDeferred`, `TamerCommandDeferred`, `ChargedTelegraphDeferred`, `EnemyTraitDeferred`, and `EnergyCapReached`.
2. Add a small DSL field for damaged-target intent, e.g. `TargetHpRule::{Any, Damaged}` on `SkillTargeting`; migrate canonical `assets/data/skills.ron` to declare the default `Any` explicitly so data remains inspectable rather than implicit.
3. Create `src/combat/action_query.rs` with pure structs/enums for `CombatQuerySnapshot`, `UnitQuerySnapshot`, `ActionQueryKind`, `ActionAffordance`, `TargetAffordance`, `ActionStatus`, `TargetStatus`, `ResourceStatus`, `ImplementationStatus`, and a `ToughnessAffordance` shape; derive `Debug`, `Clone`, `PartialEq`, `Eq` where practical.
4. Export the module from `src/combat/mod.rs` and add initial tests in `tests/action_affordance_query.rs` that compile against the public vocabulary and build inline fixture skill/snapshot data.
5. Keep the module headless and pure: no Bevy `World`, no systems, no UI/CLI imports, and no skill-ID-specific legality rules.

Failure Modes (Q5): invalid or incomplete canonical RON should fail existing skill parsing tests; missing fixture fields should be caught at compile time; unsupported query calls should initially return explicit disabled/deferred/hidden statuses rather than panic once later tasks implement behavior.

Load Profile (Q6): shared resources are immutable in-memory snapshots and skill books; per-operation cost should stay linear in units and candidate skills; 10x unit count should increase vector scanning cost, not introduce global state or ECS borrow pressure.

Negative Tests (Q7): include fixture construction for missing skill IDs, damaged vs full HP targets, and unsupported shape declarations so later tasks can assert stable reasons.
  - Files: `src/data/skills_ron.rs`, `assets/data/skills.ron`, `src/combat/action_query.rs`, `src/combat/mod.rs`, `tests/action_affordance_query.rs`
  - Verify: cargo test-dev --test action_affordance_query && cargo test-dev skills_ron

- [x] **T02: Implement DSL-driven target affordance evaluation** `est:1.5h`
  Why: The slice demo depends on callers asking which targets are legal before execution. This task implements target-level rules from `SkillDef.targeting` and proves offensive, revive, heal-like, and unsupported-shape behavior without hardcoded skill IDs.

Skills: use `tdd` to add/adjust tests before filling implementation and `verify-before-complete` before completion.

Do:
1. Add pure target-evaluation functions in `src/combat/action_query.rs`, such as `query_target_affordance(snapshot, actor_id, skill_def, target_id)` and a helper that evaluates every unit in the snapshot.
2. Enforce stable target rule priority: missing target -> `TargetNotFound`; commander -> `TargetIsCommander`; forbidden self -> `TargetIsSelf`; side mismatch -> `WrongSide`; life mismatch -> `TargetKo`/`TargetNotKo`; damaged-HP requirement with full HP -> `TargetFullHp`; unsupported non-`Single` shapes -> `Deferred(UnimplementedTargetShape)` unless the skill is hidden/deferred at the implementation layer.
3. Add tests in `tests/action_affordance_query.rs` for an implemented offensive single-target fixture: live enemy legal, ally wrong side, KO enemy target KO, commander rejected, self rejected when forbidden.
4. Add tests for an implemented revive fixture: KO ally legal, live ally target not KO, enemy wrong side.
5. Add tests for a heal-like damaged-target fixture using the new `TargetHpRule::Damaged`: damaged ally legal and full-HP ally illegal with `TargetFullHp`; do not add a real heal effect unless needed for validation.
6. Add tests for deferred row/non-single shape so targets are not reported as legal when execution is deferred.

Failure Modes (Q5): stale or missing targets must produce `TargetNotFound`; contradictory fixture data must not panic; unsupported shapes must remain deferred instead of falling through to single-target legality.

Load Profile (Q6): target evaluation scans candidate units in a snapshot; with 10x units the first bottleneck should be linear vector lookup, not mutable global state.

Negative Tests (Q7): wrong-side, KO/live mismatch, full-HP damaged-target mismatch, commander target, self target, missing target, and unsupported non-single shape.
  - Files: `src/combat/action_query.rs`, `tests/action_affordance_query.rs`
  - Verify: cargo test-dev --test action_affordance_query

- [x] **T03: Implement action and resource affordance evaluation** `est:1.5h`
  Why: Target filtering alone is not enough; UI/CLI/engine adapters need to ask whether an actor can execute a specific action right now and why not. This task implements the one-call action query around the target evaluator.

Skills: use `tdd` for action/resource cases and `verify-before-complete` before marking done.

Do:
1. Implement a pure action query function in `src/combat/action_query.rs`, e.g. `query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind) -> ActionAffordance`, that resolves `Basic`, `Skill(SkillId)`, and `Ultimate` through the actor `UnitSkills` and `SkillBook`.
2. Enforce stable action priority before target aggregation: missing actor/kit/skill -> `MissingSkill`; non-active actor -> `NotActiveUnit`; phase other than `CombatPhase::WaitingAction` -> `WrongPhase`; actor KO -> `AttackerKo`; actor stunned -> `AttackerStunned`; `SkillImplementation::Deferred`/`Hidden` -> matching action and implementation statuses; SP shortfall -> disabled resource/action with `SpShortfall`; ultimate charge below trigger -> disabled resource/action with `UltimateNotReady`; zero legal targets -> `NoValidTargets`; otherwise enabled.
3. Return resource details with current/required values for SP and ultimate readiness, while leaving future tamer/energy resource codes available but not wired in S04.
4. Include target affordances in every non-missing action result so callers can explain both action and target reasons from one pure call.
5. Add tests for `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, SP shortfall, ultimate not ready, no legal targets, implemented enabled offensive action, hidden implementation, and deferred implementation.

Failure Modes (Q5): missing actor, missing kit, missing skill book entry, stale phase, and resource shortfalls must all return explicit statuses/reasons rather than panicking or requiring display-string parsing.

Load Profile (Q6): action evaluation performs skill lookup plus target scan over the snapshot; with 10x skills/units, linear lookup is acceptable for S04 tests but should be localized for later optimization if needed.

Negative Tests (Q7): missing actor/skill, wrong active unit, wrong phase, KO/stunned actor, insufficient SP, insufficient ultimate charge, hidden/deferred skill, and empty/no-valid target lists.
  - Files: `src/combat/action_query.rs`, `tests/action_affordance_query.rs`
  - Verify: cargo test-dev --test action_affordance_query

- [x] **T04: Add toughness affordances and close the public query contract** `est:1h`
  Why: S04 also supports R085 by exposing truthful affordances for deferred/hidden mechanics and enemy-only toughness. This task finishes the query contract, proves compatibility with established S02/S03 regressions, and runs the slice-level verification suite.

Skills: use `verify-before-complete` before claiming task and slice completion; use `tdd` if adding missing toughness tests first.

Do:
1. Add query-side toughness affordance output in `src/combat/action_query.rs` that uses `exposes_toughness_affordance` / `visible_toughness` from `src/combat/toughness.rs` rather than duplicating enemy-only logic.
2. Ensure enemy units with positive toughness expose visible implemented toughness data, while ally toughness and non-positive enemy bars are hidden/disabled with `ToughnessEnemyOnly` or no visible bar according to the public status vocabulary.
3. Add/complete tests in `tests/action_affordance_query.rs` covering enemy visible toughness, ally hidden/disabled toughness, hidden self-only form-identity-like skill, and deferred unsupported target shape in the final one-call action query.
4. Run focused S02/S03 regressions to confirm the new query vocabulary did not break canonical data, revive, target-shape truthfulness, or toughness enemy-only behavior.
5. Optionally run `cargo check --features "dev windowed"` if public type/export changes ripple into feature-gated imports; fix compile issues but do not wire UI consumption in S04.

Failure Modes (Q5): duplicated toughness logic could drift from S02 helpers; public export mistakes could compile in headless but break feature-gated consumers; hidden/deferred skills could be accidentally shown as disabled instead of hidden/deferred.

Load Profile (Q6): toughness affordance computation is per-unit and cheap; at 10x units the cost remains bounded by the same snapshot scan used for target affordances.

Negative Tests (Q7): ally toughness present internally but hidden from query, enemy zero/maxless toughness not shown as breakable, hidden skill status preserved, deferred row shape status preserved, and existing regression tests remain green.
  - Files: `src/combat/action_query.rs`, `tests/action_affordance_query.rs`, `tests/toughness_enemy_only.rs`, `tests/target_shape_truthfulness.rs`, `tests/revive_semantics.rs`, `tests/skill_legality_contract_docs.rs`
  - Verify: cargo test-dev --test action_affordance_query && cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs && cargo test-dev

## Files Likely Touched

- src/data/skills_ron.rs
- assets/data/skills.ron
- src/combat/action_query.rs
- src/combat/mod.rs
- tests/action_affordance_query.rs
- tests/toughness_enemy_only.rs
- tests/target_shape_truthfulness.rs
- tests/revive_semantics.rs
- tests/skill_legality_contract_docs.rs
