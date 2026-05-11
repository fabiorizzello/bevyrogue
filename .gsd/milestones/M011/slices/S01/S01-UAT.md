# S01: Unblock action pipeline (ApplyDeferred chain) — UAT

**Milestone:** M011
**Written:** 2026-04-27T12:03:08.219Z

# UAT: S01 — Action Pipeline Lifecycle Contract

## Preconditions

- Rust toolchain per `rust-toolchain.toml`
- Working directory: repo root
- No `windowed` feature required — all tests are headless

---

## Test Case 1: Root action emits 4 lifecycle events in correct order

**Purpose:** Verify R070 — the 4-phase lifecycle is observable on the CombatEvent bus.

**Steps:**
1. Run `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test pipeline_dispatch lifecycle_root_action_emits_4_events_in_order 2>&1`
2. Observe output: `test lifecycle_root_action_emits_4_events_in_order ... ok`

**Expected:** Test passes. The bus sequence contains `OnActionDeclared { intent_kind: Basic }` → `OnActionPreApp` → (zero or more core events: `OnDamageDealt`, `OnBreak`, `OnKO`) → `OnActionApplied` → `OnActionResolved`, all with `follow_up_depth == 0`.

---

## Test Case 2: Follow-up action produces second lifecycle cycle at depth=1

**Purpose:** Verify R071 — FIFO follow-up order and depth tracking.

**Steps:**
1. Run `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test pipeline_dispatch lifecycle_follow_up_action_emits_second_cycle_with_depth_1 2>&1`
2. Observe output: `test lifecycle_follow_up_action_emits_second_cycle_with_depth_1 ... ok`

**Expected:** Test passes. After the root declared→resolved cycle (depth=0), a second declared→resolved cycle appears on the bus with `follow_up_depth == 1`.

---

## Test Case 3: SP-shortfall path still emits full lifecycle bracket

**Purpose:** Verify that lifecycle is unconditional — consumers don't need to handle missing-close events.

**Steps:**
1. Run `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --test pipeline_dispatch lifecycle_emitted_even_when_action_fails_for_sp_shortfall 2>&1`
2. Observe output: `test lifecycle_emitted_even_when_action_fails_for_sp_shortfall ... ok`

**Expected:** Test passes. Sequence contains `OnActionDeclared` → `OnActionPreApp` → `OnActionFailed` → `OnActionApplied` → `OnActionResolved`. No `OnDamageDealt` event present (no damage on failure).

---

## Test Case 4: Full integration suite is green

**Purpose:** Verify no regression from pipeline collapse across all combat subsystems.

**Steps:**
1. Run `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo test --tests --no-fail-fast 2>&1 | grep -E "^test result"`
2. Count `ok` lines vs `FAILED` lines.

**Expected:** 24 lines containing `test result: ok.`, 0 lines containing `FAILED`. Total ≥ 325 tests passed.

---

## Test Case 5: action_pipeline_system is absent from schedule

**Purpose:** Confirm dead code was removed, not just commented out.

**Steps:**
1. Run `grep -rn "action_pipeline_system" src/ 2>&1`

**Expected:** No output (zero matches). The function is fully deleted from pipeline.rs and the re-export is removed from turn_system/mod.rs.

---

## Test Case 6: CombatState has no action_stage field

**Purpose:** Confirm dead state is cleaned up.

**Steps:**
1. Run `grep -rn "action_stage\|ActionStage" src/ tests/ 2>&1`

**Expected:** No output. All references removed.

---

## Edge Cases

- **Stun path**: an actor who is stunned at the start of their turn — `OnActionDeclared` should still be emitted with `intent_kind: Basic` before the stun check aborts the action. Verified indirectly by the SP-shortfall test pattern (same emit-before-check structure in step_app).
- **Multi-follow-up FIFO**: the `combat_coherence::s_m008_s06_break_follow_up_and_ult_timing_trace` test covers two queued follow-ups (Agumon + Hackmon both triggered on OnEnemyBreak) with two separate `app.update()` calls. Each update processes exactly one FollowUpIntent. Both complete with depth=1.
