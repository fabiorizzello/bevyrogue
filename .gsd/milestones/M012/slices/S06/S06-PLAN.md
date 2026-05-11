# S06: Engine validation integration

**Goal:** Wire the pure legality query into the Bevy action pipeline as an authoritative early safety net, so forcing an illegal ActionIntent into the message bus fails with the same stable LegalityReasonCode the preflight query returns — before declaration, before mutation, before lifecycle events.
**Demo:** After this: forcing an illegal ActionIntent directly into the Bevy message bus fails with the same reason the preflight query would have returned.

## Must-Haves

- ## Must-Haves
- Pure `query_intent_legality()` helper that accepts snapshot, skill_book, actor_id, action_kind, and target_id and returns `Result<(), LegalityReasonCode>` with explicit, tested reason priority
- ECS-to-`CombatQuerySnapshot` adapter that reads from Bevy queries at runtime
- Early validation injected in `resolve_action_system()` before `step_declaration()`, emitting `OnActionFailed` with `Debug`-formatted `LegalityReasonCode` reason string
- Integration tests proving forced illegal intents produce matching reason codes between pure query and engine, with no mutation and no lifecycle events (no `OnActionDeclared`/`OnActionPreApp`/`OnActionApplied`/`OnActionResolved`)
- Active-unit compatibility: if `TurnOrder.active_unit` is `None`, treat the intent attacker as active; if `Some(other_id)`, enforce match
- SP shortfall stays in `step_app()` — not moved to early validation — to preserve existing `pipeline_dispatch` lifecycle contract
- All existing tests remain green (`cargo test-dev`)
- ## Threat Surface
- **Abuse**: None — engine validation is an internal safety net, not a user-facing API. ActionIntent is written by internal systems (AI, CLI adapter), not external input.
- **Data exposure**: None
- **Input trust**: None — all inputs are internal ECS state
- ## Requirement Impact
- **Requirements touched**: R084 (engine uses same DSL-backed legality surface), R085 (truthful UI-affecting mechanics enforced at runtime)
- **Re-verify**: `pipeline_dispatch` SP lifecycle expectations, `revive_semantics` display strings, `target_shape_truthfulness` runtime rejection
- **Decisions revisited**: None — D053/D054/D056 remain valid; this slice implements what they prescribe
- ## Proof Level
- This slice proves: integration (pure query ↔ Bevy runtime parity)
- Real runtime required: yes (Bevy app with message bus)
- Human/UAT required: no
- ## Verification
- `cargo test-dev --test action_affordance_query` — pure intent legality helper tests pass
- `cargo test-dev --test engine_legality_integration` — engine parity tests pass (new file)
- `cargo test-dev --test target_shape_truthfulness` — existing runtime rejection stays green
- `cargo test-dev --test revive_semantics` — existing revive behavior stays green
- `cargo test-dev --test pipeline_dispatch` — existing SP lifecycle contract stays green
- `cargo test-dev` — full suite green, no regressions
- ## Observability / Diagnostics
- Runtime signals: `OnActionFailed { reason }` event now emits stable `LegalityReasonCode` debug strings for target/actor failures caught by early validation; legacy display strings remain for guards in `step_app()`
- Inspection surfaces: `ActionLog` entries with reason codes; `CombatEvent` bus readable by any observer
- Failure visibility: reason code in event + log identifies exact legality failure without ambiguity
- Redaction constraints: none
- ## Integration Closure
- Upstream surfaces consumed: `action_query.rs` pure query layer (S04), `LegalityReasonCode` enum (S03/S04), `SkillBook`/`SkillDef.targeting` (S03), `RoundEnergyTracker` (S05)
- New wiring introduced in this slice: ECS snapshot adapter + early validation guard in `resolve_action_system()`
- What remains before the milestone is truly usable end-to-end: S07 (CLI/windowed affordance integration), S08 (enemy counterplay declarations), S09 (doc/data alignment)

## Proof Level

- This slice proves: Not provided.

## Integration Closure

Not provided.

## Verification

- Not provided.

## Tasks

- [x] **T01: Add pure query_intent_legality helper and reason-priority tests** `est:45m`
  # T01: Add pure query_intent_legality helper and reason-priority tests

**Slice:** S06 — Engine validation integration
**Milestone:** M012

## Description

Add a pure `query_intent_legality()` function to `src/combat/action_query.rs` that validates a specific selected intent (actor + action kind + target) against the existing query infrastructure and returns `Result<(), LegalityReasonCode>`. This helper is the bridge between the aggregate affordance API (which returns all targets with statuses) and the engine's need to validate one specific submitted intent. The function must have explicit, tested reason priority so the engine failure matches what preflight would have told the caller.

The function should:
1. Resolve the skill via `resolve_action_skill()` from the actor's kit + skill book. On failure: `MissingSkill`.
2. Check implementation status via the skill's `ImplementationStatus`. If hidden/deferred: return the corresponding reason.
3. Check actor/phase/resource status via `action_and_resource_status_for_snapshot()`. If not `ActionStatus::Available`: map to the corresponding `LegalityReasonCode` (e.g. `AttackerKo`, `AttackerStunned`, `NotActiveUnit`, `WrongPhase`, `SpShortfall`, `UltimateNotReady`). Skip `NoValidTargets` at this stage.
4. Check the specific submitted target via `target_status_for_unit()`. If not `TargetStatus::Valid`: return the target-specific reason (`WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetIsSelf`, `TargetIsCommander`, `TargetNotFound`, `UnimplementedTargetShape`).
5. Return `Ok(())` if all checks pass.

Reason priority matters: if a revive targets a live ally, the reason should be `TargetNotKo` (the specific target reason), not `NoValidTargets` (the aggregate). The priority is: missing skill > implementation > actor/resource > selected target.

## Steps

1. Add `pub fn query_intent_legality(snapshot: &CombatQuerySnapshot, skill_book: &SkillBook, actor_id: UnitId, kind: &ActionQueryKind, target_id: UnitId) -> Result<(), LegalityReasonCode>` to `src/combat/action_query.rs`.
2. Implement the 5-step validation chain described above, reusing existing helpers (`resolve_action_skill`, `action_and_resource_status_for_snapshot`, `target_status_for_unit`).
3. For `ActionStatus` → `LegalityReasonCode` mapping, add a small helper or match block. `ActionStatus::Available` and `ActionStatus::Unavailable(NoValidTargets)` both pass through to step 4 (target check). All other `Unavailable` reasons map directly.
4. Add tests in `tests/action_affordance_query.rs` covering reason priority:
   - Revive skill forced against live ally → `TargetNotKo`
   - Offensive skill forced against ally → `WrongSide`
   - Offensive skill forced against KO enemy → `TargetKo`
   - Offensive skill forced against commander → `TargetIsCommander`
   - Skill with missing target ID → `TargetNotFound`
   - SP shortfall actor → `SpShortfall`
   - Non-active actor (snapshot `is_active = false`) → `NotActiveUnit`
   - KO attacker → `AttackerKo`
   - Stunned attacker → `AttackerStunned`
   - Valid intent → `Ok(())`
5. Run `cargo test-dev --test action_affordance_query` and confirm all pass.

## Must-Haves

- [ ] `query_intent_legality()` is public and returns `Result<(), LegalityReasonCode>`
- [ ] Reason priority: missing skill > implementation > actor/resource > selected target
- [ ] `NoValidTargets` aggregate is never returned when a specific target reason exists
- [ ] At least 10 pure test cases covering the priority chain
- [ ] No Bevy dependency — function is pure over `CombatQuerySnapshot` + `SkillBook`

## Verification

- `cargo test-dev --test action_affordance_query` passes with new intent legality tests

## Inputs

- `src/combat/action_query.rs` — existing query helpers to compose
- `src/data/skills_ron.rs` — `LegalityReasonCode` enum, `SkillDef`, `ActionQueryKind`
- `tests/action_affordance_query.rs` — existing test helpers (snapshot_with, unit, actor_with_skills, skill factories)

## Expected Output

- `src/combat/action_query.rs` — new `query_intent_legality()` function added
- `tests/action_affordance_query.rs` — new intent legality test cases added
  - Files: `src/combat/action_query.rs`, `tests/action_affordance_query.rs`, `src/data/skills_ron.rs`
  - Verify: cargo test-dev --test action_affordance_query

- [x] **T02: Build ECS snapshot adapter and wire early validation into resolve_action_system** `est:1h`
  # T02: Build ECS snapshot adapter and wire early validation into resolve_action_system

**Slice:** S06 — Engine validation integration
**Milestone:** M012

## Description

Build a function that constructs a `CombatQuerySnapshot` from live Bevy ECS state, then inject it as an early validation guard in `resolve_action_system()` before `step_declaration()`. When validation fails, emit `OnActionFailed` with the `Debug`-formatted `LegalityReasonCode` and return early — no declaration, no lifecycle events, no mutation.

The adapter must read from the same Bevy resources `resolve_action_system()` already has access to: `CombatState`, `TurnOrder`, `SpPool`, and the `ResolveActorsQuery` + `energy_q` queries. It produces a `CombatQuerySnapshot` that the pure `query_intent_legality()` (from T01) can consume.

**Active-unit compatibility rule:** If `TurnOrder.active_unit` is `None`, set `is_active = true` for the intent's attacker in the snapshot. If `Some(id)`, only mark that `id` as active. This preserves existing tests that don't set `active_unit` while still enabling focused `NotActiveUnit` rejection.

**SP scoping:** Do NOT include SP shortfall in early validation. SP remains validated in `step_app()` to preserve the existing `pipeline_dispatch` lifecycle contract (SP failure currently emits lifecycle events before failing). Early validation covers: missing skill, implementation status, actor state (KO/stunned/not-active/wrong-phase), and target legality (wrong side/KO/not-KO/full-HP/self/commander/not-found/unimplemented-shape).

To implement SP exclusion cleanly: set `sp` in the snapshot to a value that will never trigger `SpShortfall` (e.g. `u32::MAX` or the skill's cost), so the pure helper skips SP without needing a special flag.

**Borrowing constraint (MEM053):** `ResolveActorsQuery` is borrowed mutably by pipeline steps. Build/clone the snapshot in a short scope before calling `step_declaration()`, so no snapshot borrow survives into the mutable pipeline path.

## Steps

1. In `src/combat/action_query.rs` (or a new `src/combat/snapshot_adapter.rs` if the import graph gets complex), add a helper function `build_snapshot_from_ecs(...)` that takes references to `CombatState`, `TurnOrder`, `SpPool`, and iterator/slice of unit data, and returns `CombatQuerySnapshot`. Map each unit's ECS components (`Unit`, `UnitSkills`, `Ko`, `Stunned`, `Commander`, `Toughness`, `UltimateCharge`, `Energy`, `RoundEnergyTracker`) to `UnitQuerySnapshot` fields.
2. Handle active-unit compatibility: check `turn_order.active_unit`. If `None`, mark intent attacker as active. If `Some(id)`, mark only that `id` as active.
3. Set `sp` to `u32::MAX` for all units in the snapshot to bypass SP validation in the pure helper.
4. In `src/combat/turn_system/mod.rs`, at the top of `resolve_action_system()` after reading the `ActionIntent`, build the snapshot in a short scope. Convert `ActionIntent` variant to `(actor_id: UnitId, target_id: UnitId, ActionQueryKind)`.
5. Load the `SkillBook` from assets (same pattern as existing code). Call `query_intent_legality(snapshot, skill_book, actor_id, kind, target_id)`.
6. On `Err(reason)`: push `LogEntry::ActionFailed { reason: format!("{reason:?}") }` to `ActionLog`, emit `CombatEvent { kind: OnActionFailed { reason: format!("{reason:?}") }, ... }` via `event_writer`, and `return` before `step_declaration()`.
7. On `Ok(())`: proceed to existing `step_declaration()` flow unchanged.
8. Run `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test pipeline_dispatch` to confirm no regressions.
9. Run `cargo test-dev` for full suite.

## Must-Haves

- [ ] `build_snapshot_from_ecs()` produces a valid `CombatQuerySnapshot` from Bevy state
- [ ] Active-unit compatibility: `None` → attacker is active; `Some(other)` → attacker not active
- [ ] SP bypassed in early validation (set to u32::MAX or equivalent)
- [ ] Early validation guard injected before `step_declaration()` in `resolve_action_system()`
- [ ] Failed validation emits `OnActionFailed` with `Debug`-formatted `LegalityReasonCode` and returns before any lifecycle event
- [ ] Snapshot borrow does not survive into mutable pipeline path
- [ ] Existing tests remain green: `pipeline_dispatch`, `revive_semantics`, `target_shape_truthfulness`

## Failure Modes

| Dependency | On error | On timeout | On malformed response |
|------------|----------|-----------|----------------------|
| ResolveActorsQuery missing unit | Skip unit in snapshot; pipeline guards catch downstream | N/A | N/A |
| SkillBook asset not loaded | Skip early validation (no skill book = no query); fall through to existing pipeline guards | N/A | N/A |

## Verification

- `cargo test-dev --test target_shape_truthfulness` — existing runtime rejection green
- `cargo test-dev --test revive_semantics` — existing revive behavior green
- `cargo test-dev --test pipeline_dispatch` — SP lifecycle contract preserved
- `cargo test-dev` — full suite green

## Observability Impact

- Signals added/changed: `OnActionFailed` reason strings now include stable `LegalityReasonCode` debug format (e.g. `"TargetNotKo"`, `"WrongSide"`) for target/actor failures caught early; legacy display strings (e.g. `"Target is not KO"`) remain for guards in `step_app()`
- How a future agent inspects this: read `ActionLog` entries or drain `CombatEvent` bus for `OnActionFailed` events
- Failure state exposed: exact `LegalityReasonCode` variant in event reason string

## Inputs

- `src/combat/action_query.rs` — `query_intent_legality()` from T01, `CombatQuerySnapshot`, `UnitQuerySnapshot`
- `src/combat/turn_system/mod.rs` — `resolve_action_system()` to modify
- `src/combat/turn_system/pipeline.rs` — `step_declaration()` interface (unchanged, for reference)
- `src/combat/state.rs` — `CombatState`, `CombatPhase`
- `src/combat/turn_order.rs` — `TurnOrder`
- `src/combat/sp.rs` — `SpPool`
- `src/combat/unit.rs` — `Unit`, `UnitId`, ECS component types
- `src/combat/events.rs` — `CombatEvent`, `CombatEventKind`
- `src/combat/log.rs` — `ActionLog`, `LogEntry`

## Expected Output

- `src/combat/action_query.rs` — new `build_snapshot_from_ecs()` adapter function (or `src/combat/snapshot_adapter.rs` if needed)
- `src/combat/turn_system/mod.rs` — early validation guard added to `resolve_action_system()`
  - Files: `src/combat/action_query.rs`, `src/combat/turn_system/mod.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/state.rs`, `src/combat/turn_order.rs`, `src/combat/sp.rs`, `src/combat/unit.rs`, `src/combat/events.rs`, `src/combat/log.rs`
  - Verify: cargo test-dev --test target_shape_truthfulness --test revive_semantics --test pipeline_dispatch && cargo test-dev

- [x] **T03: Add engine legality parity integration tests proving query-engine reason match** `est:1h`
  # T03: Add engine legality parity integration tests proving query-engine reason match

**Slice:** S06 — Engine validation integration
**Milestone:** M012

## Description

Create `tests/engine_legality_integration.rs` with integration tests that prove the S06 acceptance contract: forcing an illegal `ActionIntent` into the Bevy message bus fails with the same `LegalityReasonCode` the pure preflight query returns. Each test builds a Bevy app and a matching pure `CombatQuerySnapshot` from the same fixture data, forces an illegal intent, and asserts:

1. The pure `query_intent_legality()` returns the expected `Err(reason_code)`.
2. The engine emits exactly one `OnActionFailed` event whose reason string matches `format!("{reason_code:?}")`.
3. No lifecycle events are emitted (`OnActionDeclared`, `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`).
4. No mutation occurs (target HP/KO state unchanged after `app.update()`).

**Key Bevy test patterns (from project memories):**
- Use `MessageCursor<CombatEvent>` and drain after each `app.update()` (MEM059) to avoid stale-event re-counting.
- Build inline `SkillBook` fixtures (not canonical `skills.ron`) to isolate test scenarios.
- Use the existing test helper patterns from `tests/action_affordance_query.rs` for snapshot construction and from `tests/target_shape_truthfulness.rs` / `tests/revive_semantics.rs` for Bevy app construction.

**Test cases to include:**

| Case | Skill type | Intent target | Expected reason code |
|------|-----------|---------------|---------------------|
| Revive live ally | revive (target_life: KoOnly) | live ally | `TargetNotKo` |
| Offensive KO enemy | offensive (target_side: Enemy) | KO enemy | `TargetKo` |
| Offensive ally | offensive (target_side: Enemy) | live ally | `WrongSide` |
| Commander target | offensive | commander unit | `TargetIsCommander` |
| KO attacker | offensive | live enemy | `AttackerKo` |
| Stunned attacker | offensive | live enemy | `AttackerStunned` |
| Non-active actor | offensive, with TurnOrder.active_unit = Some(other) | live enemy | `NotActiveUnit` |

Optional stretch cases (if time permits): `TargetNotFound` (nonexistent target ID), `TargetIsSelf` (self-targeting with `allow_self_target: false`).

## Steps

1. Create `tests/engine_legality_integration.rs`.
2. Add test helper functions for Bevy app construction: build a minimal `App` with combat plugins, spawn units with appropriate components, insert `CombatState`, `SpPool`, `TurnOrder`, `SkillBookHandle`, and load a `SkillBook` asset. Mirror patterns from `tests/revive_semantics.rs` or `tests/target_shape_truthfulness.rs`.
3. Add a helper to build a matching `CombatQuerySnapshot` from the same fixture values used in the Bevy app.
4. Add a `drain_events()` helper using `MessageCursor<CombatEvent>` that collects events from one `app.update()` cycle.
5. Implement each test case from the table above. Each test:
   a. Builds Bevy app + pure snapshot from shared fixture.
   b. Calls `query_intent_legality()` on the snapshot; asserts expected `Err(code)`.
   c. Writes `ActionIntent` to the app's message bus.
   d. Calls `app.update()` and drains events.
   e. Asserts exactly one `OnActionFailed` with reason matching `format!("{code:?}")`.
   f. Asserts no lifecycle events.
   g. Asserts target entity state unchanged (HP, Ko component).
6. Run `cargo test-dev --test engine_legality_integration` and confirm all pass.
7. Run `cargo test-dev` for full suite green.

## Must-Haves

- [ ] At least 7 integration test cases covering the table above
- [ ] Each test proves query↔engine reason parity (same `LegalityReasonCode`)
- [ ] Each test proves no lifecycle events emitted for rejected intents
- [ ] Each test proves no mutation (HP/KO unchanged)
- [ ] Uses `MessageCursor<CombatEvent>` drain pattern (MEM059)
- [ ] Full `cargo test-dev` passes

## Negative Tests

- **Malformed inputs**: nonexistent target ID → `TargetNotFound`
- **Error paths**: KO attacker, stunned attacker, non-active actor — all reject before mutation
- **Boundary conditions**: commander target (special unit type), self-targeting

## Verification

- `cargo test-dev --test engine_legality_integration` — all parity tests pass
- `cargo test-dev` — full suite green, no regressions

## Inputs

- `src/combat/action_query.rs` — `query_intent_legality()` from T01, `CombatQuerySnapshot` builder helpers
- `src/combat/turn_system/mod.rs` — early validation guard from T02 (runtime behavior under test)
- `src/data/skills_ron.rs` — `LegalityReasonCode`, `SkillDef`, skill factory patterns
- `tests/revive_semantics.rs` — reference for Bevy app construction patterns in tests
- `tests/target_shape_truthfulness.rs` — reference for runtime rejection test patterns
- `tests/action_affordance_query.rs` — reference for pure snapshot builder helpers

## Expected Output

- `tests/engine_legality_integration.rs` — new integration test file with 7+ parity tests
  - Files: `tests/engine_legality_integration.rs`
  - Verify: cargo test-dev --test engine_legality_integration && cargo test-dev

## Files Likely Touched

- src/combat/action_query.rs
- tests/action_affordance_query.rs
- src/data/skills_ron.rs
- src/combat/turn_system/mod.rs
- src/combat/turn_system/pipeline.rs
- src/combat/state.rs
- src/combat/turn_order.rs
- src/combat/sp.rs
- src/combat/unit.rs
- src/combat/events.rs
- src/combat/log.rs
- tests/engine_legality_integration.rs
