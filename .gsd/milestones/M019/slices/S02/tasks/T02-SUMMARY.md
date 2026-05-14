---
id: T02
parent: S02
milestone: M019
key_files:
  - src/combat/state.rs
  - src/combat/resolution.rs
key_decisions:
  - Floor division (hp_max * pct) / 100 chosen for predictable integer arithmetic (no ceil) — consistent with existing revive formula pattern
  - Heal KO no-op returns sp_ok=true with empty events, no OnActionFailed emitted — matches S02 ROADMAP no-op policy
  - Heal branch placed before revive/damage guard in apply_effects so heal skills bypass the 'Target is KO' early-return path
duration: 
verification_result: passed
completed_at: 2026-05-14T08:42:33.825Z
blocker_discovered: false
---

# T02: Added apply_heal_only helper and wired Effect::Heal into apply_effects for Single/SelfOnly targets with KO no-op and OnHealed event emission

**Added apply_heal_only helper and wired Effect::Heal into apply_effects for Single/SelfOnly targets with KO no-op and OnHealed event emission**

## What Happened

Added `heal_pct: u32` field to `ResolvedAction` in state.rs. Added `skill_heal_pct` extractor (mirrors `skill_revive_pct` pattern) in resolution.rs and wired it into `resolve_action`. Added `apply_heal_only` helper: KO guard first → silent return; otherwise computes `(hp_max * pct) / 100` with i64 widening, caps to `hp_max - hp_current`, increments `hp_current`, emits `OnHealed { amount, hp_after }`. In `apply_effects`, inserted a heal branch before the revive/damage-path KO guard: if `heal_pct > 0` and target is KO, returns `sp_ok=true` with no events (no-op policy); if alive, calls `apply_heal_only` and collects the event. The `AllAllies` arms in `resolve_targets` and `target_shape_is_executable_now` were already present from T01. Updated 8 test files that constructed `ResolvedAction` struct literals to include `heal_pct: 0`.

## Verification

cargo check passed (0 errors). cargo test passed: all test suites green, 0 failures. Existing baseline test trace identity preserved — no Heal skills in fixture data so no JSONL output changed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | pass | 1960ms |
| 2 | `cargo test` | 0 | pass — all suites green | 15000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/combat/state.rs`
- `src/combat/resolution.rs`
