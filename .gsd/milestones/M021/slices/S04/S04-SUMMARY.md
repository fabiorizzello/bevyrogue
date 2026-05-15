---
id: S04
parent: M021
milestone: M021
provides:
  - (none)
requires:
  []
affects:
  []
key_files:
  - src/combat/api/signal.rs
  - src/combat/api/intent.rs
  - src/combat/api/applier.rs
  - src/combat/api/blueprint_state.rs
  - src/combat/api/passive_runner.rs
  - src/combat/api/runner_common.rs
  - src/combat/api/event_bridge.rs
  - src/combat/plugin.rs
  - src/combat/kernel.rs
  - tests/passive_kitsune_grace.rs
  - tests/blueprint_signal_dispatcher.rs
key_decisions:
  - Use owned String for Signal and CombatKernelTransition::Blueprint payload names to keep JSON/serde round-trips straightforward.
  - Keep bridge tests isolated with a minimal App rather than full CombatPlugin when unrelated plugins need additional Messages initialization.
  - Register the kernel taxonomy entry at plugin build time so the first update tick can safely emit the bridged signal.
patterns_established:
  - Reactive behavior is driven by SignalBus + PassiveRunner, not by special-case ult logic in the turn pipeline.
  - Blueprint events remain observable through CombatEvent so JSONL logging can reuse the existing serde path.
  - Debug assertions guard unregistered signals while release builds follow the warn-and-drop policy.
observability_surfaces:
  - CombatEvent::OnKernelTransition { Blueprint }
  - JSONL round-trip for kernel transitions
  - debug_assert path for unregistered signal emission
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-15T14:34:37.114Z
blocker_discovered: false
---

# S04: SignalBus + PassiveRunner + Ult instant + Intent::BlueprintSignal dispatcher

**Renamon kitsune_grace now triggers end-to-end through SignalBus/PassiveRunner, the kernel Blueprint transition round-trips through JSONL, and unregistered signal emission is guarded.**

## What Happened

S04 closed the reactive layer around Blueprint signals. The typed SignalBus/SignalPayload/SignalTaxonomy foundation was already in place from T01, and the slice now verifies the full path: CombatEvent::UltimateUsed is bridged into a kernel Blueprint signal, the intent applier dispatches it onto SignalBus while emitting CombatKernelTransition::Blueprint for JSONL, PassiveRunner consumes matching signals through the shared beat helpers, and the Renamon kitsune_grace passive advances AV by 10% on an ally ult. Negative guards were also confirmed: self-ult and enemy-ult do not fire the passive, and unregistered signal emission is guarded by the taxonomy path. The capstone integration test plus inline unit coverage exercised queue ordering, taxonomy registration, dispatcher emission, runner trigger matching, signal-cascade circuit-breaking, and serde round-trip behavior. Verification was run end-to-end after assembly and all required checks passed.

## Verification

Verified with cargo test --test passive_kitsune_grace, cargo test, cargo check, cargo check --features windowed, and rg -n "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/ returning 0 hits. The capstone test passed, the full suite passed, both headless and windowed checks passed, and the guard grep confirmed no leaked franchise-specific names in the API layer.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

T04’s dispatcher tests use a minimal App setup rather than full CombatPlugin bootstrap to avoid unrelated startup panics from plugins that require extra Messages registration. The runtime behavior under test is unchanged.

## Known Limitations

None identified in this slice.

## Follow-ups

None.

## Files Created/Modified

None.
