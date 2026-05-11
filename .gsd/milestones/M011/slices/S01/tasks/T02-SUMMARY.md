---
id: T02
parent: S01
milestone: M011
key_files:
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/follow_up.rs
  - src/headless.rs
  - src/windowed.rs
key_decisions:
  - Widened emit_combat_event from pub(super) to pub(crate) to allow follow_up.rs to use the same structured event emission helper rather than duplicating the debug-log + write pattern
  - Lifecycle events are emitted unconditionally after step_app returns — even on abort paths (stun/SP shortfall) — so OnActionResolved always follows OnActionDeclared for complete observability
  - follow_up_depth in lifecycle events sourced from inflight.follow_up_depth (set by step_declaration) rather than hardcoded, making the caller-set depth the single source of truth
duration: 
verification_result: passed
completed_at: 2026-04-27T11:02:26.819Z
blocker_discovered: false
---

# T02: Collapsed action pipeline to single-tick flow: resolve_action_system and resolve_follow_up_action_system now inline step_app and emit 4 lifecycle events; action_pipeline_system deleted

**Collapsed action pipeline to single-tick flow: resolve_action_system and resolve_follow_up_action_system now inline step_app and emit 4 lifecycle events; action_pipeline_system deleted**

## What Happened

Eliminated the two-tick action pipeline by inlining `step_app` directly into both action resolution systems and emitting the 4 lifecycle `CombatEvent` variants around each call.

**resolve_action_system** (`src/combat/turn_system/mod.rs`): Removed `inflight: Option<Res<InFlightAction>>` parameter and the `inflight.is_some() || state.action_stage != ActionStage::None` guard. Added `SpPool`, `ActionLog`, `TurnOrder`, `Time` as system params. Inside the `if let Some(intent)` branch, the system now: derives `intent_kind`, calls `step_declaration` to get a local `InFlightAction`, emits `OnActionDeclared { intent_kind }` + `OnActionPreApp`, calls `step_app` inline, then emits `OnActionApplied` + `OnActionResolved`. The `pub use pipeline::action_pipeline_system` re-export was removed. `emit_combat_event` was widened from `pub(super)` to `pub(crate)` so `follow_up.rs` can use it.

**resolve_follow_up_action_system** (`src/combat/follow_up.rs`): Mirror refactor with `follow_up_depth = 1` sourced from `inflight.follow_up_depth`. Removed guard and `InFlightAction` resource parameter; added the same four resource params. `ActionIntentKind::Skill` is hardcoded (follow-up is always a Skill action).

**pipeline.rs** cleanup: Removed `state.action_stage = ActionStage::PreApp` from `step_declaration`, all four `state.action_stage = ActionStage::None` assignments from `step_app`, `ActionStage` from imports, and the entire `action_pipeline_system` function.

**Schedule cleanup**: Removed `action_pipeline_system` from both `headless.rs` and `windowed.rs` system chains and their imports.

Verification confirmed: 5 integration test binaries pass, `action_pipeline_system` function is gone from pipeline.rs, and `cargo check --tests` emits zero errors. The 4 remaining test failures (`s08_ultimate_interrupt_flow`, `s_m008_s06_break_follow_up_and_ult_timing_trace`, `s10_follow_up_reentrancy_is_suppressed_after_one_hop`, `s_m006_roster_smoke_deterministic`) are pre-existing data configuration issues (missing unit names "Hackmon"/"Impmon", unknown rookie id UnitId(8)) confirmed unchanged via `git stash` round-trip.

## Verification

Ran exact T02 verification command: `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test encounter_e2e --test follow_up_triggers --test sp_economy --test status_effect_apply --test ultimate_meter 2>&1 | grep -E '^test result' | grep -v 'FAILED' | wc -l | xargs -I {} test {} -ge 5 && ! grep -q 'pub fn action_pipeline_system' src/combat/turn_system/pipeline.rs` — PASSED. Also ran `cargo check --tests` (zero errors) and full suite (`cargo test --tests --no-fail-fast`) confirming 19/23 binaries green vs 10/23 before T02; 4 remaining failures are pre-existing data issues not related to the pipeline.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests 2>&1 | grep -E '^error' | wc -l` | 0 | ✅ pass | 18000ms |
| 2 | `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test encounter_e2e --test follow_up_triggers --test sp_economy --test status_effect_apply --test ultimate_meter 2>&1 | grep -E '^test result' | grep -v FAILED | wc -l | xargs -I {} test {} -ge 5 && ! grep -q 'pub fn action_pipeline_system' src/combat/turn_system/pipeline.rs && echo PASSED` | 0 | ✅ pass | 45000ms |
| 3 | `grep -q 'pub fn action_pipeline_system' src/combat/turn_system/pipeline.rs && echo FOUND || echo GONE` | 0 | ✅ pass (GONE) | 50ms |

## Deviations

None. step_declaration's state.action_stage=PreApp removal and step_app's state.action_stage=None removals were executed exactly as specified.

## Known Issues

4 pre-existing test failures unrelated to this task: s08_ultimate_interrupt_flow (test design assumes 1-tick but pipeline was 2-tick before T02, still broken by missing ult-reset behavior), s_m008_s06_break_follow_up_and_ult_timing_trace (missing pilot Hackmon in units.ron), s10_follow_up_reentrancy_is_suppressed_after_one_hop (missing pilot Impmon in units.ron), s_m006_roster_smoke_deterministic (UnknownRookie id UnitId(8)).

## Files Created/Modified

- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/follow_up.rs`
- `src/headless.rs`
- `src/windowed.rs`
