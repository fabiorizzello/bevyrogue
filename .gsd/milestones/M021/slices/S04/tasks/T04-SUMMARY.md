---
id: T04
parent: S04
milestone: M021
key_files:
  - src/combat/api/event_bridge.rs
  - src/combat/api/mod.rs
  - src/combat/plugin.rs
key_decisions:
  - Used minimal App setup (add_message + SignalBus only) rather than CombatPlugin in tests to avoid Messages-not-initialized panic from DorumonPlugin — follows turn_system/tests.rs pattern.
  - Registered kernel taxonomy entry at plugin build() time (not finish()) so it is available for the first update tick.
duration: 
verification_result: mixed
completed_at: 2026-05-15T14:20:23.119Z
blocker_discovered: false
---

# T04: Added CombatEvent::UltimateUsed → SignalBus bridge system (combat_event_to_signal_system) with kernel taxonomy registration and 3 passing inline unit tests.

**Added CombatEvent::UltimateUsed → SignalBus bridge system (combat_event_to_signal_system) with kernel taxonomy registration and 3 passing inline unit tests.**

## What Happened

Created src/combat/api/event_bridge.rs with `combat_event_to_signal_system`: reads `MessageReader<CombatEvent>`, matches on `CombatEventKind::UltimateUsed { unit_id }`, and pushes `Signal::Blueprint { owner: "kernel", name: "ult_used", payload: UnitTarget(unit_id), cast_id }` onto `SignalBus`. Wired into plugin.rs with ordering `intent_applier → combat_event_to_signal_system → passive_dispatch_system`. Registered `("kernel", "ult_used")` in `SignalTaxonomy` at plugin build time so the debug_assert! gate in the applier does not fire for kernel-emitted signals. Added `pub mod event_bridge` and `pub use event_bridge::combat_event_to_signal_system` to api/mod.rs. Three inline unit tests cover: ult-event produces exactly one kernel signal with correct fields; non-ult events produce no signal; two ult events produce two signals. Tests use a minimal App (add_message::<CombatEvent> + SignalBus only) following the pattern in turn_system/tests.rs rather than CombatPlugin, to avoid the Messages-not-initialized panic from DorumonPlugin. `TacticalCyclePhase::UltInstant` has zero occurrences in src/ — D010 satisfied by bridge only, no new phase variant introduced.

## Verification

cargo test --lib -- combat::api::event_bridge: 3/3 passed. cargo check: clean (headless). cargo check --features windowed: clean. rg TacticalCyclePhase::UltInstant src/: 0 hits.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --lib -- combat::api::event_bridge` | 0 | 3 tests passed | 1550ms |
| 2 | `cargo check` | 0 | clean (warnings only) | 130ms |
| 3 | `cargo check --features windowed` | 0 | clean (warnings only) | 130ms |
| 4 | `rg "TacticalCyclePhase::UltInstant" src/` | 1 | 0 hits — no new phase variant introduced | 50ms |

## Deviations

Test helper uses minimal App instead of full CombatPlugin as the task plan suggested — CombatPlugin would panic at first update because DorumonPlugin's MessageReader<CombatEvent> requires Messages<CombatEvent> to be registered externally, which CombatPlugin does not do itself. The bridge behavior is identical; only the test harness differs.

## Known Issues

None.

## Files Created/Modified

- `src/combat/api/event_bridge.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
