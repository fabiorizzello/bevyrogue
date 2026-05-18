---
id: T02
parent: S04
milestone: M021
key_files:
  - src/combat/api/applier.rs
  - src/combat/api/blueprint_state.rs
  - src/combat/api/signal.rs
  - src/combat/kernel.rs
  - src/combat/events.rs
  - src/combat/toughness.rs
  - tests/blueprint_signal_dispatcher.rs
key_decisions:
  - Use String in Signal payload for serde compatibility
duration: 
verification_result: passed
completed_at: 2026-05-15T11:11:21.637Z
blocker_discovered: false
---

# T02: Wired BlueprintSignal and SetBlueprintState dispatchers in intent_applier and added CombatKernelTransition::Blueprint.

**Wired BlueprintSignal and SetBlueprintState dispatchers in intent_applier and added CombatKernelTransition::Blueprint.**

## What Happened

Wired Intent::BlueprintSignal and Intent::SetBlueprintState in the intent_applier. BlueprintSignal now enqueues on the SignalBus (after verifying against SignalTaxonomy) and emits a CombatEventKind::OnKernelTransition { transition: CombatKernelTransition::Blueprint }. SetBlueprintState writes to the new BlueprintState resource. Fixed compilation issues where CombatEvent and its dependencies were missing Deserialize impls, and addressed lifetime issues in the Blueprint transition variant by using String instead of &'static str. Verified with integration tests and cargo check across feature gates.

## Verification

cargo test --test blueprint_signal_dispatcher passed; cargo check headless and windowed passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test blueprint_signal_dispatcher` | 0 | ✅ pass | 4500ms |

## Deviations

Used String instead of &'static str in Signal and CombatKernelTransition::Blueprint to satisfy serde Deserialize requirements. Added serde derives to DamageKind to fix compilation.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/applier.rs`
- `src/combat/api/blueprint_state.rs`
- `src/combat/api/signal.rs`
- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/toughness.rs`
- `tests/blueprint_signal_dispatcher.rs`
