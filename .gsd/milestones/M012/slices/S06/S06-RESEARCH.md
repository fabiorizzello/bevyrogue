# S06 Research: Engine validation integration

## Summary

S06 should wire the already-built pure legality/query layer into the authoritative Bevy action pipeline. The key acceptance contract is: if an illegal `ActionIntent` is written directly to the Bevy message bus, the engine rejects it before mutation and reports the same stable `LegalityReasonCode` that the preflight target/action query exposes.

This slice primarily advances **R084** (same DSL-backed legality surface for UI/CLI/tests/engine) and supports **R085** (truthful UI-affecting mechanics; no late hidden/half-implemented affordances). The current code has the pure query vocabulary and tests, but the runtime pipeline still uses older guard strings and effect-derived checks in `pipeline.rs` / `resolution.rs`.

## Skill / process notes

- `verify-before-complete` applies to the implementation handoff: do not mark this slice complete from compile-only evidence. S06 needs fresh proof that forced illegal intents fail with matching query reason codes and no mutation.
- Relevant installed skills from prompt: `test` is useful for adding/running focused integration tests; `debug-like-expert` may help if Bevy message ordering/regression failures appear.
- Skill discovery for Bevy found optional external skills, not installed: `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert` (108 installs) and `npx skills add laurigates/claude-plugins@bevy-ecs-patterns` (25 installs). These are relevant to Bevy ECS mechanics but not required; the existing project patterns are enough.

## Implementation landscape

### Pure query layer: `src/combat/action_query.rs`

Existing public surface:

- `CombatQuerySnapshot` and `UnitQuerySnapshot`: immutable data model for query consumers.
- `ActionQueryKind::{Basic, Skill(&SkillId), Ultimate}`.
- Status enums: `ActionStatus`, `TargetStatus`, `ResourceStatus`, plus `ImplementationStatus`.
- `TargetAffordance` and `ActionAffordance` include per-target status, resource details, implementation state, and toughness affordance.
- Public functions:
  - `query_action_affordance(snapshot, skill_book, actor_id, kind)`
  - `query_target_affordance(snapshot, actor_id, skill_def, target_id)`
  - `query_all_target_affordances(...)`
  - resource helpers from S05.

Important behavior:

- `target_status_for_unit()` already implements DSL-backed target rules: shape, side, life, self, commander, damaged/full HP.
- `action_and_resource_status_for_snapshot()` already implements actor/resource rules: active unit, phase, KO/stunned, SP, ultimate, and aggregate no-target handling.
- `resolve_action_skill()` enforces kit membership and `SkillBook` lookup.
- There is **no ECS snapshot adapter** yet. All query tests build `CombatQuerySnapshot` manually.
- There is **no selected-intent validation helper** yet. `query_action_affordance()` aggregates target legality; for forced bus validation S06 should validate the submitted `target_id` specifically.

Recommended seam: add a small pure helper in `action_query.rs`, e.g. `query_intent_legality(...) -> Result<(), LegalityReasonCode>` or `intent_legality_reason(...) -> Option<LegalityReasonCode>`, that combines action/resource/implementation status with the submitted target's `TargetAffordance`. This prevents `pipeline.rs` from pattern-matching query output itself and keeps the reason priority testable without Bevy.

Reason priority should be explicit and tested. Suggested priority:

1. Missing skill / missing actor kit.
2. Implementation hidden/deferred reason.
3. Actor/phase/resource state failures except aggregate `NoValidTargets`.
4. Submitted target status reason (`WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetIsCommander`, `TargetIsSelf`, `TargetNotFound`, `UnimplementedTargetShape`).
5. Aggregate `NoValidTargets` only when there is no more specific selected-target reason.

This matters because revive-on-live-ally has aggregate `NoValidTargets` when every ally is live, but the selected target preflight reason is `TargetNotKo`; S06 acceptance wants the engine failure to match the preflight reason for the forced intent.

### Runtime pipeline: `src/combat/turn_system/mod.rs`

`resolve_action_system()` currently:

1. Reads one `ActionIntent` from `Messages<ActionIntent>`.
2. Calls `pipeline::step_declaration(...)`.
3. Emits lifecycle events `OnActionDeclared` and `OnActionPreApp` only if declaration succeeds.
4. Calls `pipeline::step_app(...)`.
5. Emits `OnActionApplied` and `OnActionResolved`.

It has access to the resources S06 needs for engine preflight: `CombatState`, `SpPool`, `TurnOrder`, `Assets<SkillBook>`, `SkillBookHandle`, `ResolveActorsQuery`, and `energy_q`. The cleanest runtime integration point is at the start of `resolve_action_system()`, before `step_declaration()`, because this is where an injected bus intent can be rejected before lifecycle and before mutation.

Caveat: existing tests often create an app and write `ActionIntent` directly without setting `TurnOrder.active_unit`. If the engine snapshot maps `UnitQuerySnapshot.is_active` strictly to `turn_order.active_unit == Some(unit.id)`, many older tests may start failing with `NotActiveUnit`. To preserve compatibility while still enabling active-unit validation, consider:

- compatibility mode: if `turn_order.active_unit.is_none()`, mark the intent attacker as active; if it is `Some(id)`, enforce exact match; or
- update all tests/fixtures to set `TurnOrder.active_unit` before writing intents.

The first option is lower-risk for S06 and still lets a focused test prove stale actor rejection by setting `active_unit = Some(other_id)`.

### Runtime pipeline guards: `src/combat/turn_system/pipeline.rs`

Current validation is split across declaration and app:

- `step_declaration()` resolves the skill and rejects unimplemented target shapes with string reasons like `UnimplementedTargetShape:Row`, before lifecycle.
- `step_app()` has duplicated guard blocks for self/no-hit actions and normal actor-target pairs. It emits display strings such as `Attacker is stunned`, `Attacker is KO`, `Target is KO`, `Target is not KO`, `Target is a Commander`, and `SP shortfall`.
- `apply_effects()` in `resolution.rs` repeats some checks and may also emit `OnActionFailed` display strings.

S06 should not try to remove every legacy guard in one pass. Treat query-backed validation as an authoritative early safety net, then leave lower-level guards as defensive fallback. After early query validation, those older paths should be mostly unreachable for normal illegal target/action cases, but they may still protect edge cases or future direct function tests.

Potential conflict: `tests/pipeline_dispatch.rs` currently expects SP shortfall to emit `OnActionDeclared` and `OnActionPreApp` before `OnActionFailed`, then still emit Applied/Resolved. A stricter S06 pre-declaration rejection for SP would break that contract. Planner should decide whether S06 changes this lifecycle contract or scopes tests around target/actor legality only. If changing SP behavior, update/replace the pipeline dispatch expectation deliberately.

### Skill data and reason codes: `src/data/skills_ron.rs`

`LegalityReasonCode` already contains the S06 vocabulary needed for engine parity: `WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetIsSelf`, `TargetIsCommander`, `NoValidTargets`, `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, `SpShortfall`, `UltimateNotReady`, `TargetNotFound`, `UnimplementedTargetShape`, etc.

`SkillDef.targeting` is the canonical source of target side/life/shape/self/hp rules (per MEM076/MEM083). Engine validation should not infer revive/offense legality from `Effect` or skill IDs. It should resolve the skill from the actor kit + skill book, then ask the query helper.

### Existing tests to reuse or mirror

- `tests/action_affordance_query.rs`: pure query coverage for offensive, revive, damaged ally, SP, ultimate, KO/stunned, commander, implementation hidden/deferred, resource caps. Useful as the source of expected reason codes.
- `tests/target_shape_truthfulness.rs`: current runtime rejection before mutation/lifecycle for Row/AllEnemies. These should stay green; after S06 they can be tightened to assert exact reason code strings if desired.
- `tests/revive_semantics.rs`: current runtime revive success/failure uses display strings (`Target is not KO`, `Target is KO`). S06 can add new tests rather than rewriting this immediately, or migrate assertions to stable code strings if compatibility permits.
- `tests/pipeline_dispatch.rs`: watch for SP lifecycle expectations as noted above.
- Memory gotcha MEM059: use a `MessageCursor<CombatEvent>` and drain after each `app.update()` when asserting Bevy message output.

## Natural work seams

1. **Pure selected-intent legality helper**
   - File: `src/combat/action_query.rs`
   - Add a helper that accepts snapshot, skill book, actor id, action kind, target id and returns `Ok(())` or the selected stable `LegalityReasonCode`.
   - Add pure unit/integration tests in `tests/action_affordance_query.rs` for priority cases: revive live ally => `TargetNotKo`, offensive ally => `WrongSide`, offensive KO target => `TargetKo`, missing target => `TargetNotFound`, SP shortfall => `SpShortfall`, non-active actor => `NotActiveUnit`.

2. **ECS snapshot adapter for engine validation**
   - Best location: `src/combat/action_query.rs` if kept pure-ish with Bevy query types avoided, or `src/combat/turn_system/pipeline.rs` / `mod.rs` if tied to Bevy `Query` signatures.
   - Inputs needed: `CombatState.phase`, `TurnOrder.active_unit`, shared `SpPool.current`, actor/target/all units from `ResolveActorsQuery`, `UltimateCharge`, `UnitSkills`, `Toughness`, `Ko`, `Stunned`, `Commander`, and optionally `Energy`/`RoundEnergyTracker`.
   - For `UnitQuerySnapshot.sp`, use `SpPool.current` for all units or at least the actor; SP is a team/shared resource in runtime but per-actor in the snapshot API.
   - For `is_ko`, prefer `Ko` component or `Unit::is_ko()`; many tests apply `Ko` via pipeline, but HP <= 0 is also meaningful.
   - For `is_active`, choose and document the compatibility rule around `TurnOrder.active_unit.is_none()`.

3. **Runtime validation and failure emission**
   - File: `src/combat/turn_system/mod.rs` is the likely top-level integration point.
   - Convert `ActionIntent` to `(actor_id, target_id, ActionQueryKind)`.
   - Build snapshot, resolve skill book, call selected-intent helper.
   - On failure: push `LogEntry::ActionFailed { reason: format!("{reason:?}") }`, emit `CombatEventKind::OnActionFailed { reason: format!("{reason:?}") }`, and return before declaration/app.
   - Keep lower-level guards for fallback, but S06 tests should prove the early query-backed path is used by asserting no lifecycle events and no mutation for target/action illegal forced intents.

4. **Focused engine parity tests**
   - New file suggestion: `tests/engine_legality_integration.rs`.
   - Use inline `SkillBook` fixtures like existing tests, not canonical data unless specifically testing canonical behavior.
   - Build both a pure `CombatQuerySnapshot` and a Bevy app from the same fixture; compare the pure selected-target reason to the engine `OnActionFailed.reason`.
   - Cases to include for acceptance:
     - Revive-like skill forced against live ally: query selected target `TargetNotKo`; engine emits `TargetNotKo`; target HP/KO state unchanged; no declaration/preapp/applied/resolved.
     - Offensive skill forced against KO enemy: `TargetKo`; no mutation.
     - Offensive skill forced against ally: `WrongSide`; no mutation.
     - Commander target: `TargetIsCommander`; no mutation.
     - Optional resource/state: SP shortfall `SpShortfall`, stale actor `NotActiveUnit`, stunned/KO attacker.
   - Use `MessageCursor<CombatEvent>` to drain current-frame events (MEM055/MEM059).

## Risks and watch-outs

- **Lifecycle regression risk:** Existing SP failure test expects lifecycle closure after SP failure. Decide intentionally whether SP joins early validation in S06 or remains a later app-stage guard until lifecycle expectations are updated.
- **Active-unit compatibility:** Most historical tests do not set `TurnOrder.active_unit`. Strict active-unit enforcement will cause broad failures. Compatibility rule or fixture updates are required.
- **Reason string compatibility:** Existing tests assert display strings such as `Target is KO`. S06 wants stable reason code parity. Prefer adding new reason-code tests first, then migrate old display-string tests only if necessary.
- **Borrowing complexity:** `resolve_action_system()` already passes `actors` mutably into pipeline. Build/clone the snapshot in a short scope before calling `step_declaration()` so no query borrow survives.
- **No skill-ID hardcoding:** All runtime validation must flow through actor `UnitSkills` + `SkillBook` + `SkillDef.targeting`; do not special-case revive/heal/offense skill IDs.

## Recommended plan for planner

Build/prove S06 in this order:

1. Add selected-intent pure helper and pure tests. This locks down reason priority without Bevy complexity.
2. Add engine snapshot construction with a minimal compatibility rule for `TurnOrder.active_unit`.
3. Integrate early validation in `resolve_action_system()` and emit reason-code strings via `Debug` formatting of `LegalityReasonCode`.
4. Add `tests/engine_legality_integration.rs` comparing query reason to engine failure reason for forced illegal intents and checking no mutation/lifecycle.
5. Run focused tests, then `cargo test-dev`.

## Verification commands

Minimum focused verification after implementation:

```bash
cargo test-dev --test action_affordance_query
cargo test-dev --test engine_legality_integration
cargo test-dev --test target_shape_truthfulness
cargo test-dev --test revive_semantics
cargo test-dev --test pipeline_dispatch
```

Final S06 verification before completing the slice:

```bash
cargo test-dev
```

If implementation touches windowed/UI code unexpectedly, also run:

```bash
cargo check --features "dev windowed"
```

## Sources inspected

- `src/combat/action_query.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/resolution.rs`
- `src/data/skills_ron.rs`
- `src/combat/state.rs`
- `src/combat/turn_order.rs`
- `src/combat/energy.rs`
- `src/combat/sp.rs`
- `src/combat/ultimate.rs`
- `src/combat/unit.rs`
- `src/combat/log.rs`
- `tests/action_affordance_query.rs`
- `tests/target_shape_truthfulness.rs`
- `tests/revive_semantics.rs`
- `tests/pipeline_dispatch.rs`
- `docs/skill_legality_contract.md`
- GSD memories MEM055, MEM059, MEM076, MEM077, MEM081, MEM083, MEM084, MEM085, MEM086
