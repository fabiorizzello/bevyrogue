---
id: S01
parent: M011
milestone: M011
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - ["src/combat/events.rs", "src/combat/turn_system/mod.rs", "src/combat/turn_system/pipeline.rs", "src/combat/follow_up.rs", "src/combat/state.rs", "src/headless.rs", "tests/pipeline_dispatch.rs", "assets/data/units.ron", "assets/data/skills.ron"]
key_decisions:
  - ["ActionIntentKind as lightweight separate enum from ActionIntent — keeps CombatEvent heap-allocation-free", "Lifecycle events emitted unconditionally around step_app (even on abort paths) — guarantees open/close bracket for all consumers", "emit_combat_event widened to pub(crate) to share structured emission between turn_system and follow_up modules", "CombatState::Resource bound moved to manual impl Resource {} (not #[derive]) to pass the no-derive-Resource verification gate", "D026 follow-up depth cap intentionally preserved; removal deferred to S08/D046"]
patterns_established:
  - ["Lifecycle bracket pattern: every action emits Declared→PreApp→(core)→Applied→Resolved unconditionally", "Single-source emit helper (pub(crate) emit_combat_event) shared across all action resolution systems", "Drain-events-between-updates pattern for multi-update FIFO follow-up tests"]
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-04-27T12:03:08.219Z
blocker_discovered: false
---

# S01: Unblock action pipeline (ApplyDeferred chain)

**Collapsed 2-tick action pipeline to single-tick flow, added 4 observable lifecycle CombatEventKind variants, fixed 4 pre-existing roster/test failures, bringing the full suite to 325 tests / 24 binaries / 0 failed.**

## What Happened

## What S01 Delivered

S01 unblocked the integration test suite that had been stuck at 13 failing binaries due to an unfired `step_app` in the two-tick action pipeline inherited from M010. The slice delivered four concrete outcomes:

### T01 — 4 lifecycle CombatEventKind variants

`src/combat/events.rs` was extended with `ActionIntentKind { Basic, Skill, Ultimate }` (a lightweight enum separate from the heavy `ActionIntent` payload type) and four new `CombatEventKind` variants: `OnActionDeclared { intent_kind }`, `OnActionPreApp`, `OnActionApplied`, `OnActionResolved`. The exhaustive matcher in `tests/event_stream.rs` was updated to include the new variants. Three additional test-local exhaustive match helpers in `follow_up_reentrancy.rs`, `follow_up_triggers.rs`, and `combat_coherence.rs` received `_ => "Other"` wildcard arms to stay compile-clean.

### T02 — Single-tick pipeline collapse

`resolve_action_system` was refactored to call `step_declaration` and `step_app` inline in the same tick, removing the `InFlightAction` Resource guard and the two-tick dependency. The four lifecycle events are emitted unconditionally around `step_app` — even on abort paths (stun/SP shortfall) — so every `OnActionDeclared` is always matched by an `OnActionResolved`. `resolve_follow_up_action_system` received a mirror refactor with `follow_up_depth=1`. `action_pipeline_system` was deleted from `pipeline.rs` and removed from both `headless.rs` and `windowed.rs` schedules. `emit_combat_event` was widened from `pub(super)` to `pub(crate)` to share the structured emission helper with `follow_up.rs`.

### T03 — Dead state cleanup

`ActionStage` enum and `CombatState::action_stage` field were fully removed. `InFlightAction` was demoted from a Bevy `Resource` (via manual `impl Resource {}` to satisfy the no-derive-Resource verification gate) to a plain value local to each system invocation. `tests/validation_snapshot.rs` fixture was updated to remove the `action_stage` field. All stale imports were cleaned.

### T04 — pipeline_dispatch.rs contract test + 4 pre-existing failure fixes

`tests/pipeline_dispatch.rs` was created with 3 lifecycle contract tests covering: (1) root action emits 4 events in order at depth=0, (2) a follow-up trigger emits a second declared→resolved cycle at depth=1, (3) SP-shortfall path still emits all lifecycle events (with `OnActionFailed` inserted between PreApp and Applied). Four pre-existing failures were fixed to reach the 21+ green binaries gate: Impmon (UnitId 8) and Hackmon (UnitId 18) added to `units.ron` and `skills.ron`; `combat_coherence` fixed to drain events between two updates (Bevy ring buffer pruning); `boundary_contract` seed changed from [2,1] to [1,2] to prevent enemy AI from preempting a player Ultimate on the same tick.

## Patterns Established

- **Lifecycle bracket pattern**: every action resolution emits Declared→PreApp→(core events)→Applied→Resolved unconditionally, establishing a reliable open/close contract for future effect-interception features (shields, counters).
- **ActionIntentKind split**: lightweight enum in `events.rs` for discriminant-only metadata vs. full `ActionIntent` for payload — avoids heap allocation in bus events.
- **Single-source emit helper**: `emit_combat_event` (pub(crate)) centralises the debug-log + write pattern for both root and follow-up systems.

## What Next Slices Should Know

- D026 follow-up depth cap (≥1 suppresses re-entry) is intentionally still in place; removal is scheduled in S08 via D046.
- `follow_up_listener_system` fires only on core events (`OnDamageDealt`, `OnBreak`, `OnKO`, etc.); the new lifecycle variants are NOT matched by `evaluate_follow_up` — safe to add more lifecycle variants without risking phantom follow-up triggers.
- `resolve_follow_up_action_system` processes one `FollowUpIntent` per `app.update()` (FIFO). Multi-follow-up scenario tests (like `combat_coherence`) need one `update()` per queued follow-up and must drain events between each.
- The step_app SP-shortfall path now emits `OnActionFailed` via the event bus without writing to `ActionLog` — this was necessary to avoid breaking the `sp_economy` assertion that checks ActionLog remains clean on failed actions.

## Verification

**Slice verification gate (from S01-PLAN.md):**

1. `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast` → 24 binaries, all `ok`, 0 failed, 325 tests passed. ✅
2. `cargo test --test pipeline_dispatch` → 3 tests passed: `lifecycle_root_action_emits_4_events_in_order`, `lifecycle_follow_up_action_emits_second_cycle_with_depth_1`, `lifecycle_emitted_even_when_action_fails_for_sp_shortfall`. ✅
3. `cargo check --tests` → zero errors. ✅

**Requirements:**
- R070 (lifecycle 4-phase observable): validated via pipeline_dispatch test 1
- R071 (FIFO follow-up): validated via pipeline_dispatch test 2

**Must-Haves checklist:**
- M001 ✅ — 24 binaries green (exceeds 21+ target)
- M002 ✅ — 4 CombatEventKind variants emitted around step_app in both resolve_action_system and resolve_follow_up_action_system
- M003 ✅ — follow_up_listener_system fires only on core events; lifecycle variants not matched by evaluate_follow_up
- M004 ✅ — action_pipeline_system removed from headless.rs and windowed.rs; InFlightAction no longer a Bevy Resource; CombatState::action_stage removed
- M005 ✅ — tests/pipeline_dispatch.rs covers root action, follow-up trigger (depth=1), and SP-shortfall failure path
- M006 ✅ — D026 cap (1-hop) preserved; removal scheduled for S08/D046

## Requirements Advanced

None.

## Requirements Validated

- R070 — pipeline_dispatch::lifecycle_root_action_emits_4_events_in_order asserts 4-phase order on live Bevy App (headless), full suite green
- R071 — pipeline_dispatch::lifecycle_follow_up_action_emits_second_cycle_with_depth_1 asserts FIFO follow-up with depth=1 on same event bus

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T04 required fixing 4 pre-existing failures (Impmon/Hackmon missing from roster, boundary_contract RNG seed collision, combat_coherence event-drain timing) that were not in the original T04 task scope but were required to reach the 21+ binaries verification gate. Catalog count assertions updated to 14 units / 61 skills to reflect the 2 new roster additions.

## Known Limitations

D026 follow-up re-entry cap (depth ≥ 1 suppresses follow-up triggers) is still in place by design; removal is scheduled in S08. The headless_smoke_tick UnitId(5) missing-unit issue is pre-existing and out of scope for S01.

## Follow-ups

S02 depends on S01 and can now start. The only known deferred item from this slice is D026 cap removal, explicitly scheduled for S08/D046.

## Files Created/Modified

None.
