---
id: T01
parent: S01
milestone: M011
key_files:
  - src/combat/events.rs
  - tests/event_stream.rs
  - tests/follow_up_reentrancy.rs
  - tests/follow_up_triggers.rs
  - tests/combat_coherence.rs
key_decisions:
  - Defined ActionIntentKind as a separate lightweight enum in events.rs rather than reusing ActionIntent, keeping lifecycle metadata free of heap-allocated payload fields
  - Added wildcard arms to 3 test-local exhaustive match helpers rather than removing them, preserving their diagnostic JSON output for known variants
duration: 
verification_result: passed
completed_at: 2026-04-27T10:49:32.298Z
blocker_discovered: false
---

# T01: Added ActionIntentKind enum and 4 lifecycle CombatEventKind variants (OnActionDeclared/PreApp/Applied/Resolved); patched 3 exhaustive matchers in test files

**Added ActionIntentKind enum and 4 lifecycle CombatEventKind variants (OnActionDeclared/PreApp/Applied/Resolved); patched 3 exhaustive matchers in test files**

## What Happened

Extended `src/combat/events.rs` with a new `ActionIntentKind { Basic, Skill, Ultimate }` enum (serializable, Debug+Clone+PartialEq+Eq) co-located with `CombatEventKind`, then appended the 4 required lifecycle variants: `OnActionDeclared { intent_kind: ActionIntentKind }`, `OnActionPreApp`, `OnActionApplied`, and `OnActionResolved`. Updated the exhaustive allowed-set matcher in `tests/event_stream.rs` (lines 200-210) to include all 4 new variants so the test remains valid when T02 starts emitting them. Added an `ActionIntentKind` import to the test file with a `let _ = ActionIntentKind::Basic` sentinel to suppress the unused-import warning until T02 activates the variants. Discovered that three additional test files (`follow_up_reentrancy.rs`, `follow_up_triggers.rs`, `combat_coherence.rs`) contained local `event_kind_json` / `trace_kind_json` helper functions that were exhaustive matches with no wildcard arm — each received a `_ => \"{\\\"kind\\\":\\\"Other\\\"}\"` fallback arm. Both `jsonl_logger.rs` and `log.rs` were confirmed safe (no exhaustive match on `CombatEventKind`). `follow_up.rs` match at line 142 already had a `_ => false` wildcard. `cargo check --tests` exits clean with no errors after all changes.

## Verification

Ran `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests` — zero errors. Confirmed `grep -q 'OnActionDeclared' src/combat/events.rs` and `grep -q 'OnActionResolved' tests/event_stream.rs` both return exit 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm cargo check --tests 2>&1 | grep -E '^error' | wc -l` | 0 | ✅ pass | 18500ms |
| 2 | `grep -q 'OnActionDeclared' src/combat/events.rs && echo OK` | 0 | ✅ pass | 10ms |
| 3 | `grep -q 'OnActionResolved' tests/event_stream.rs && echo OK` | 0 | ✅ pass | 10ms |

## Deviations

Three test files (follow_up_reentrancy.rs, follow_up_triggers.rs, combat_coherence.rs) contained exhaustive CombatEventKind matches not mentioned in the task plan. Fixed by adding _ wildcard arms — consistent with the task plan's instruction to add `_ => {}` fallbacks where needed.

## Known Issues

none

## Files Created/Modified

- `src/combat/events.rs`
- `tests/event_stream.rs`
- `tests/follow_up_reentrancy.rs`
- `tests/follow_up_triggers.rs`
- `tests/combat_coherence.rs`
