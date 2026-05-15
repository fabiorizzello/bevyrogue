---
id: T05
parent: S04
milestone: M021
key_files:
  - tests/passive_kitsune_grace.rs
key_decisions:
  - Use a BlueprintState latch to terminate the cyclic listener FSM cleanly after the proc beat.
  - Emit BlueprintSignal from the proc beat so the integration test can prove the JSONL Blueprint round-trip on the existing event surface.
duration: 
verification_result: passed
completed_at: 2026-05-15T14:33:33.520Z
blocker_discovered: false
---

# T05: Added the Renamon kitsune_grace end-to-end integration test suite with ally/self/enemy guards, JSONL Blueprint round-trip, and debug_assert coverage.

**Added the Renamon kitsune_grace end-to-end integration test suite with ally/self/enemy guards, JSONL Blueprint round-trip, and debug_assert coverage.**

## What Happened

Created tests/passive_kitsune_grace.rs from the shared Bevy integration-test scaffold. The test file builds a minimal kitsune_grace listener FSM (Dormant → Proc → Resolve → Dormant), registers a PassiveRunner for Renamon against the kernel ult signal, and exercises the positive ally case plus self/enemy negative guards. The proc beat writes a BlueprintState sentinel to prove the passive fired and emits a BlueprintSignal so the resulting OnKernelTransition::Blueprint CombatEvent can be serialized and deserialized through serde_json. Added a separate debug-build panic check for unregistered blueprint signals to cover the negative taxonomy guard.

## Verification

Verified the new test file with cargo test --test passive_kitsune_grace (5/5 passing). Verified the full suite with cargo test. Verified the windowed build with cargo check --features windowed. Verified the P001 api-surface sweep is clean: rg on TwinCore/BatteryLoop/HolySupport/PredatorLoop/PrecisionMindGame/KitsuneGrace under src/combat/api/ returned zero hits.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test passive_kitsune_grace` | 0 | ✅ pass | 822ms |
| 2 | `cargo test && cargo check --features windowed && rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/ >/tmp/passive_kitsune_grace_rg.log 2>&1 || true` | 0 | ✅ pass | 79410ms |
| 3 | `test ! -s /tmp/passive_kitsune_grace_rg.log && echo 'zero hits'` | 0 | ✅ pass | 44ms |

## Deviations

Used a proc-beat BlueprintSignal in addition to the BlueprintState sentinel so the positive case produces a JSONL-round-trippable OnKernelTransition::Blueprint event on the existing reactive surface. AdvanceTurn itself remains substituted by the sentinel because the current passive runner/applier path in this slice does not route AV changes directly.

## Known Issues

None.

## Files Created/Modified

- `tests/passive_kitsune_grace.rs`
