---
id: S06
parent: M012
milestone: M012
provides:
  - A canonical engine/runtime parity contract for illegal intents.
  - A reusable ECS-to-query snapshot adapter pattern for future CLI/windowed affordance work.
  - Stable reason-code output for downstream UI enable/disable/explain logic.
requires:
  []
affects:
  []
key_files:
  - src/combat/action_query.rs
  - src/combat/turn_system/mod.rs
  - tests/engine_legality_integration.rs
  - src/combat/turn_system/tests.rs
  - tests/action_affordance_query.rs
  - tests/revive_semantics.rs
  - tests/target_shape_truthfulness.rs
key_decisions:
  - Implemented engine validation as a pure-query parity check rather than duplicating legality rules inside the Bevy pipeline.
  - Preserved SP shortfall in step_app() by bypassing SP from the early guard so the lifecycle contract did not change for that failure mode.
  - Standardized failure assertions on Debug-form LegalityReasonCode strings instead of legacy prose messages.
  - Used a readonly ECS snapshot adapter plus a transient buffer to avoid borrow conflicts with the mutable action pipeline.
patterns_established:
  - One legality source of truth must feed both preflight and runtime rejection.
  - Bevy CombatEvent parity tests should drain with MessageCursor after each update.
  - Priority-ordered legality checks should surface the most specific actionable reason before generic aggregates.
observability_surfaces:
  - `OnActionFailed` now carries stable Debug-form reason strings for illegal intents.
  - ActionLog entries mirror the same canonical reason code surface.
  - Rejected intents now leave a clear event trace without any lifecycle events.
drill_down_paths:
  - .gsd/milestones/M012/slices/S06/tasks/T01-SUMMARY.md
  - .gsd/milestones/M012/slices/S06/tasks/T02-SUMMARY.md
  - .gsd/milestones/M012/slices/S06/tasks/T03-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-01T13:19:41.975Z
blocker_discovered: false
---

# S06: S06

**Engine validation now uses the same pure legality contract as preflight, so illegal ActionIntent payloads are rejected by the Bevy bus before declaration, mutation, or lifecycle events, with stable Debug-form reason codes.**

## What Happened

S06 closed the last gap between the pure legality surface and the live combat pipeline. We added a pure `query_intent_legality()` helper that resolves a specific intent in the same priority order used by preflight, then built `build_snapshot_from_ecs()` to translate live Bevy ECS state into a transient `CombatQuerySnapshot` without holding borrows across the mutable execution path. `resolve_action_system()` now performs an early legality guard before `step_declaration()`, emitting `OnActionFailed` with the canonical `Debug` string of `LegalityReasonCode` and returning early when an intent is illegal.

The runtime contract is now parity-checked in two directions. The pure query tests prove the intended reason precedence and target-specific failures, while the new `engine_legality_integration` suite forces illegal `ActionIntent` values into the Bevy message bus and verifies the engine returns the same reason code, emits exactly one failure event, produces no lifecycle events, and leaves combat state unchanged. We also preserved the existing SP lifecycle behavior by explicitly bypassing SP shortfall in the early guard so that failure still occurs in `step_app()` as before.

This slice established the implementation pattern the remaining UI-facing slices should follow: one pure legality source of truth, one short-lived ECS snapshot adapter, and one machine-readable failure code path that both headless tests and future CLI/windowed affordance code can rely on. The slice also updated the existing turn-system assertions to the new canonical reason-code strings so the codebase no longer mixes prose failure messages with machine-readable legality output.

## Verification

Fresh verification run in this session:
- `cargo test-dev --test engine_legality_integration` ✅ 7/7 parity tests passed
- `cargo test-dev` ✅ full suite passed (131 lib tests + 132 main tests + all integration tests, including `engine_legality_integration`, `action_affordance_query`, `pipeline_dispatch`, `revive_semantics`, `target_shape_truthfulness`, `resource_caps`, `toughness_enemy_only`, `skill_legality_contract_docs`, and the rest)

Key acceptance points observed in the outputs:
- Forced illegal intents reject before lifecycle events and before mutation
- `OnActionFailed` carries stable `Debug` reason strings like `TargetNotKo`, `WrongSide`, `AttackerStunned`
- `TurnOrder.active_unit = None` preserves compatibility by treating the attacker as active
- SP shortfall still passes through the legacy lifecycle path rather than early validation

## Requirements Advanced

- R084 — The engine now rejects illegal intents through the same DSL-backed legality surface that preflight uses, and parity is verified by integration tests.
- R085 — UI-relevant legality truthfulness is now enforced at runtime for action execution, preventing false affordances from surviving into the combat pipeline.

## Requirements Validated

None.

## New Requirements Surfaced

- R084
- R085

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

SP shortfall still resolves in the legacy step_app() path by design; it is not part of early validation.

## Follow-ups

S07 should consume the same query surface for CLI/windowed affordances so UI code never re-implements legality checks.

## Files Created/Modified

None.
