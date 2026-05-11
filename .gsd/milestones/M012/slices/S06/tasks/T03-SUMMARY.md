---
id: T03
parent: S06
milestone: M012
key_files:
  - tests/engine_legality_integration.rs
  - src/combat/turn_system/tests.rs
key_decisions:
  - Used inline skill-book fixtures per case so the pure snapshot and engine guard share the exact same legality data.
  - Asserted the debug form of `LegalityReasonCode` as the canonical failure string to keep the engine guard and pure query in parity.
  - Used `MessageCursor<CombatEvent>` drain-after-update to avoid stale-event re-counting across Bevy message frames.
duration: 
verification_result: passed
completed_at: 2026-05-01T13:16:32.483Z
blocker_discovered: false
---

# T03: Added 7 engine legality parity tests that lock rejected intents to the canonical LegalityReasonCode and preserve combat state.

**Added 7 engine legality parity tests that lock rejected intents to the canonical LegalityReasonCode and preserve combat state.**

## What Happened

Created `tests/engine_legality_integration.rs` to prove the engine/pure-query legality contract across seven rejection cases: revive-on-live-ally, KO target, wrong side, commander target, KO attacker, stunned attacker, and non-active actor. Each case now builds a minimal Bevy app plus an inline `SkillBook` fixture, derives a matching `CombatQuerySnapshot`, asserts the pure `query_intent_legality()` result, injects an `ActionIntent` into the Bevy message bus, drains `CombatEvent` with a `MessageCursor`, and verifies exactly one `OnActionFailed` with the canonical `LegalityReasonCode` debug string while no lifecycle events or target-state mutations occur. The full suite exposed three stale turn-system assertions that still expected legacy prose failure strings, so I updated those existing tests to the new canonical reason-code form to keep the codebase aligned with the T02 engine guard behavior.

## Verification

Verified with `cargo test-dev --test engine_legality_integration` and `cargo test-dev`. The targeted integration test suite passed 7/7 cases, and the full workspace suite passed after updating the stale turn-system assertions to the canonical debug-form legality reason strings.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test engine_legality_integration` | 0 | ✅ pass | 8300ms |
| 2 | `cargo test-dev` | 0 | ✅ pass | 10100ms |

## Deviations

Updated three existing `src/combat/turn_system/tests.rs` assertions to expect canonical `LegalityReasonCode` debug strings (`TargetKo`, `TargetNotKo`, `AttackerStunned`) because the engine early-guard contract now emits those stable codes rather than human-readable prose.

## Known Issues

None.

## Files Created/Modified

- `tests/engine_legality_integration.rs`
- `src/combat/turn_system/tests.rs`
