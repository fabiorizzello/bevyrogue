---
estimated_steps: 55
estimated_files: 9
skills_used: []
---

# T02: Build ECS snapshot adapter and wire early validation into resolve_action_system

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

## Inputs

- ``src/combat/action_query.rs` — query_intent_legality() from T01, CombatQuerySnapshot, UnitQuerySnapshot`
- ``src/combat/turn_system/mod.rs` — resolve_action_system() to modify with early validation guard`
- ``src/combat/turn_system/pipeline.rs` — step_declaration() interface for reference`
- ``src/combat/state.rs` — CombatState, CombatPhase`
- ``src/combat/turn_order.rs` — TurnOrder with active_unit field`
- ``src/combat/sp.rs` — SpPool`
- ``src/combat/unit.rs` — Unit, UnitId, Ko, Stunned, Commander components`
- ``src/combat/events.rs` — CombatEvent, CombatEventKind::OnActionFailed`
- ``src/combat/log.rs` — ActionLog, LogEntry::ActionFailed`

## Expected Output

- ``src/combat/action_query.rs` — new build_snapshot_from_ecs() adapter function`
- ``src/combat/turn_system/mod.rs` — early validation guard in resolve_action_system()`

## Verification

cargo test-dev --test target_shape_truthfulness --test revive_semantics --test pipeline_dispatch && cargo test-dev
