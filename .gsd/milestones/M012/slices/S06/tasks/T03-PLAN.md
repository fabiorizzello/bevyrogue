---
estimated_steps: 62
estimated_files: 1
skills_used: []
---

# T03: Add engine legality parity integration tests proving query-engine reason match

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

## Inputs

- ``src/combat/action_query.rs` — query_intent_legality() from T01, build_snapshot_from_ecs() from T02`
- ``src/combat/turn_system/mod.rs` — early validation guard from T02 (runtime behavior under test)`
- ``src/data/skills_ron.rs` — LegalityReasonCode, SkillDef, targeting types`
- ``tests/revive_semantics.rs` — reference for Bevy app construction patterns`
- ``tests/target_shape_truthfulness.rs` — reference for runtime rejection test patterns`
- ``tests/action_affordance_query.rs` — reference for pure snapshot builder helpers`

## Expected Output

- ``tests/engine_legality_integration.rs` — new integration test file with 7+ query-engine parity tests`

## Verification

cargo test-dev --test engine_legality_integration && cargo test-dev
