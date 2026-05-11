# M011/S01 — Research: Unblock action pipeline (ApplyDeferred chain)

**Date:** 2026-04-27

## Summary

The M010 hand-off (`.gsd/M010-HANDOFF.md`) attributes the 13 failing test binaries to a Bevy
"deferred-commands-not-flushed" bug in the chain `resolve_action_system → action_pipeline_system`,
and proposes adding an explicit `ApplyDeferred` sync point. **That diagnosis is wrong.** Bevy
0.18.1 ships `AutoInsertApplyDeferredPass` (`bevy_ecs/src/schedule/auto_insert_apply_deferred.rs`)
with `ScheduleBuildSettings::auto_insert_apply_deferred = true` by default
(`bevy_ecs/src/schedule/schedule.rs:1581`). Any chained pair of systems where the upstream system
contains a deferred parameter (`Commands`, `MessageWriter`) and the downstream system orders after
it gets a sync point inserted automatically. The wire-up done in M010
(`src/headless.rs:124-139`, `src/windowed.rs:103-118`) is therefore structurally adequate.

The actual root cause is one line up the stack: **every test schedule still registers only
`resolve_action_system`** (`grep -L action_pipeline_system tests/*.rs` confirms zero matches in
`tests/`). M010 split a single imperative system into two — declaration in `step_declaration`
(still inside `resolve_action_system`) and apply in `step_app` (now inside the unregistered
`action_pipeline_system`). Tests therefore never run the apply phase, no `BasicHit`/`Break`/`Ko`
log entries are written, and assertions like `tests/encounter_e2e.rs:209` fail. Same pattern in
3 lib tests in `src/combat/turn_system/tests.rs` (`resolve_action_system_rejects_*`).

The slice description for S01 says: *"After this: cargo test passa al 100%; nuovo integration
tests/pipeline_dispatch.rs esercita declared→pre→apply→resolve end-to-end via CombatEvent bus"*.
R070 wants the **lifecycle observable** (so future shield/interrupt code can listen), not
necessarily the multi-system Bevy split. The handoff itself notes that v5.3 MVP has no
mid-action interrupt mechanic (Charged Attack is enemy boss-only, resolved pre-action via Danger
Window detection). The cleanest unblock is **Option C from the handoff**: collapse the pipeline
back into a single Bevy system, keep `step_declaration`/`step_app` as separate functions for
clarity, and make the lifecycle phases observable by emitting `CombatEvent` variants
(`OnActionDeclared`, `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`) on stage transitions.

## Recommendation

**Collapse pipeline to single system + lifecycle as emitted events.**

1. Drop `InFlightAction` as a Bevy `Resource`. Pass it as a local value between `step_declaration`
   and `step_app` inside one system tick.
2. Drop `action_pipeline_system` entirely (and its registration in `headless.rs` + `windowed.rs`).
3. Refactor `resolve_action_system` (and the symmetric `resolve_follow_up_action_system`) to call
   `step_declaration` → emit `OnActionDeclared`/`OnActionPreApp` → `step_app` → emit
   `OnActionApplied`/`OnActionResolved`, all in one frame.
4. Keep `ActionStage` enum **only** as a label on the emitted events (drop the `action_stage` field
   on `CombatState` once nothing else reads it; or keep purely as last-known marker for debug).
5. Add `tests/pipeline_dispatch.rs` that asserts the four lifecycle events are emitted in order
   for a basic action and that follow-up actions emit a second declared→applied cycle with
   `follow_up_depth = 1`.

Why not Option A (keep the split, add explicit `ApplyDeferred`):
- Auto-insert already does this; the split therefore costs zero per-frame perf but adds two
  resource-shaped touchpoints (`commands.insert_resource(InFlightAction)` then
  `commands.remove_resource::<InFlightAction>()`) plus the `Option<Res<InFlightAction>>` guard.
- The split was justified by an interruption window the MVP does not use.
- D021 explicitly recorded "Imperative pipeline... NIENTE multi-stage Bevy system schedule".

Why not Option B (full collapse, no lifecycle events): R070 is `active` and its `why` is "sblocca
abilità difensive (scudi)". Even though MVP doesn't ship a shield, emitting the lifecycle on the
event bus pays the cost once and lets future code subscribe without re-plumbing. Cost is ~4 new
`CombatEventKind` variants and 4 `event_writer.write` calls.

This satisfies R070 (lifecycle phases tracked + observable) and R071 (FIFO follow-up; already
implemented in `follow_up.rs:205-280` via `MessageReader` iteration order — unchanged by S01).

## Implementation Landscape

### Key Files

- `src/combat/turn_system/pipeline.rs` — currently holds `step_declaration` (lines 25-59),
  `step_app` (lines 61-222), and `action_pipeline_system` (lines 224-261). Collapse target:
  delete `action_pipeline_system`, keep the two helpers; have callers chain them inline. Note
  the `action_stage = ActionStage::PreApp` write at line 55 and `ActionStage::None` at line 220 —
  these become irrelevant once the resource is gone.
- `src/combat/turn_system/mod.rs` — `resolve_action_system` (lines 134-162) currently calls
  `step_declaration` and stores the result via `commands.insert_resource(new_inflight)`. Refactor
  to call `step_app` directly with the local `new_inflight`. The `inflight.is_some()` guard
  (line 144) becomes dead code; drop the param.
- `src/combat/follow_up.rs` — `resolve_follow_up_action_system` (lines 283-327) has the **same**
  insert_resource pattern; must be refactored in lockstep. `follow_up_listener_system`
  (lines 205-280) is unchanged — it consumes `CombatEvent` and emits `FollowUpIntent`, no
  pipeline coupling.
- `src/combat/state.rs` — `InFlightAction` (lines 53-58) becomes a non-Resource struct (or moves
  next to `step_app` as a private call-site type). `ActionStage` (lines 17-25): only `None` and
  `PreApp` are written; `Declaration`, `App`, `Resolution` are dead variants (compiler warns).
  After collapse, the `action_stage` field on `CombatState` (line 66) has no remote readers —
  drop it.
- `src/combat/events.rs` — add 4 new `CombatEventKind` variants for lifecycle phases. Confirm
  `follow_up_listener_system::evaluate_follow_up` does not pattern-match the new variants (so
  lifecycle events don't accidentally fire follow-ups).
- `src/headless.rs:122-140` and `src/windowed.rs:103-118` — drop `action_pipeline_system` from
  the Update chain; remove the `action_pipeline_system` import.
- `tests/encounter_e2e.rs:36`, `tests/follow_up_triggers.rs`, `tests/sp_economy.rs`,
  `tests/status_effect_apply.rs`, `tests/ultimate_meter.rs`, `tests/boundary_contract.rs`,
  `tests/combat_coherence.rs`, `tests/event_stream.rs`, `tests/follow_up_reentrancy.rs`,
  `tests/patamon_revive.rs`, `tests/revive_semantics.rs`, `tests/roster_smoke.rs`,
  `tests/status_effect_integration.rs` — register only `resolve_action_system` today; **no
  changes required** after the collapse (collapse moves the apply logic back behind that single
  system, which is exactly what they expect).
- `src/combat/turn_system/tests.rs` — same pattern; the 3 currently-failing
  `resolve_action_system_rejects_*` tests will pass without modification.
- `tests/pipeline_dispatch.rs` (new) — asserts the lifecycle event sequence on the
  `CombatEvent` bus.

### Build Order

1. **Add lifecycle CombatEventKind variants** (`OnActionDeclared { intent_kind }`,
   `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`). Stub `match` arms in any consumer
   that exhaustively matches the enum (`jsonl_logger.rs`, `log.rs`, follow-up evaluator). Compile
   clean. **Why first:** lets every later step emit events without churn.
2. **Collapse `resolve_action_system`** to call `step_app` directly with the local
   `InFlightAction`. Emit the four lifecycle events around the call. Drop the `inflight` param
   and the `action_stage` guard. **Why next:** unblocks 12+ test binaries in one move; verifiable
   with `cargo test --test encounter_e2e`.
3. **Mirror the collapse in `resolve_follow_up_action_system`.** Same shape, `follow_up_depth = 1`.
   **Why:** unblocks `follow_up_triggers`, `follow_up_reentrancy`.
4. **Drop `action_pipeline_system`** from `headless.rs`, `windowed.rs`, and from the
   `pipeline.rs` exports/`mod.rs` re-export. Confirm `cargo check` clean.
5. **Drop `InFlightAction: Resource`** + `CombatState::action_stage` field if unused; clean up
   `ActionStage::Declaration|App|Resolution` dead variants.
6. **Add `tests/pipeline_dispatch.rs`** — see Verification.
7. **Run full suite.** Expected: 21/21 binary green, all `turn_system::tests` green.

### Verification Approach

```bash
# Per-step quick check during build:
CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests
CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test encounter_e2e

# Final gate:
CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast 2>&1 \
  | grep -E "^test result"
# Expect: every line "ok"; total ≈ 21 pass / 0 fail.
```

`tests/pipeline_dispatch.rs` (new) should assert, for a single-action scenario:
- A `CombatEvent` sequence containing `OnActionDeclared` → `OnActionPreApp` → (zero or more core
  events like `OnDamageDealt`, `OnBreak`) → `OnActionApplied` → `OnActionResolved`, in that
  positional order, with `follow_up_depth = 0`.
- For a follow-up trigger scenario (e.g., Patamon `OnAllyLowHp`): a second declared→resolved
  cycle appears after the first, with `follow_up_depth = 1`.
- All four lifecycle variants are emitted exactly once per action attempt (even when the action
  is rejected for SP shortfall: declared+resolved still emitted, `OnActionFailed` between).

## Constraints

- **Bevy 0.18.1 message API:** `MessageReader`/`MessageWriter` (not `EventReader`/`EventWriter`);
  `Messages<T>::get_cursor().read(...)` for test introspection. The codebase already uses this
  shape (`tests/encounter_e2e.rs:19-27`).
- **Headless-first (D015):** the new `pipeline_dispatch` test must build a minimal `App` without
  `windowed` features (pattern: `tests/encounter_e2e.rs` minimal app, no DefaultPlugins).
- **No per-Digimon code (D020):** lifecycle event emission lives in the generic pipeline path,
  never branched on unit id.
- **Determinism (R019):** lifecycle events are emitted in a fixed order per action; no RNG, no
  wall-clock; ordering assertions are stable.
- **Cranelift ICE workaround:** the `dev` profile uses `cranelift` and ICEs on this codebase; all
  test runs must set `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm` (per handoff §"Comandi utili").

## Common Pitfalls

- **Do not gate `follow_up_listener_system` on lifecycle events.** It must continue to fire on the
  *core* events (`OnDamageDealt`, `OnKO`, `OnBreak`, `OnAllyLowHp`, `OnEnemyKill`, …). Add a
  guard or rely on the existing `FollowUpTrigger` enum not having variants for the lifecycle
  kinds.
- **`commands.remove_resource::<InFlightAction>()` removal** — once `InFlightAction` is no longer
  a `Resource`, every site that inserted/removed it must compile-clean. `pipeline.rs:253` and
  `pipeline.rs:257` are the current sites.
- **`jsonl_logger_system`** writes a JSONL event per `CombatEvent`. Adding 4 new variants
  multiplies its output; check that snapshot fixtures (if any) tolerate this. `tests/event_stream`
  asserts on event sequence — verify the new lifecycle events are not in conflict with existing
  expected ordering, or add explicit filtering.
- **The 3 failing `resolve_action_system_rejects_*` lib tests** assert on a `BasicHit`/`ActionFailed`
  log entry produced by `step_app`. After collapse they pass; do not "fix" them by gutting the
  assertion.
- **Headless smoke run is broken for an unrelated reason** (`headless_smoke_tick` calls
  `order.insert_out_of_queue(UnitId(5))` in `headless.rs:299` for a unit that doesn't exist in
  the new MVP roster, producing infinite "missing unit" warnings and never reaching
  `WaitingAction`). This is **out of scope for S01** — S01 is gated on `cargo test`, not
  `cargo run` smoke output. Note for whoever picks up CLI work in S04.

## Open Risks

- The new lifecycle events may overlap semantically with `OnActionFailed` (which is emitted
  inside `step_app` for stun, SP shortfall, etc.). Resolution: `OnActionResolved` is always
  emitted after the (possibly aborted) `step_app`; `OnActionFailed` remains the *reason* event
  emitted inside the apply phase. The `pipeline_dispatch` test should pin this contract.
- If `CombatState::action_stage` turns out to be read by an as-yet-unseen consumer (e.g. UI
  panel), removing the field could break `windowed`. `grep -rn "action_stage" src/` confirms only
  `pipeline.rs` and `state.rs` touch it today, so it's safe to drop.
- 21 binaries currently fail in 13 binaries' worth of asserts; the unblock is mechanical, but if
  any binary asserts on the *absence* of an event kind (e.g. event_stream snapshot), the new
  lifecycle variants will need to be filtered out. Quick scan of `tests/event_stream.rs` is on
  the executor's checklist.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy 0.18 ECS | `sickn33/antigravity-awesome-skills@bevy-ecs-expert` | installed (global) |

## Sources

- Bevy 0.18.1 source: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.1/src/schedule/auto_insert_apply_deferred.rs:21-25` — confirms `AutoInsertApplyDeferredPass` is added by default and inserts sync points on chained edges where the upstream system has `Deferred` params.
- Bevy 0.18.1 source: `bevy_ecs-0.18.1/src/schedule/schedule.rs:1557,1581` — `auto_insert_apply_deferred` defaults to `true`.
- Reproducer: `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test encounter_e2e` → fails on `tests/encounter_e2e.rs:209` (assert `BasicHit` present), confirming `step_app` never executes because `action_pipeline_system` is not in the test schedule.
- `grep -L action_pipeline_system tests/*.rs` → all 23 test binaries; zero register the system.
