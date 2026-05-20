---
id: T03
parent: S03
milestone: M002
key_files:
  - src/ui/phase_strip.rs
  - tests/phase_strip_readonly.rs
key_decisions:
  - Exposed `read_latest_observed_combat_beat` as a read-only Bevy system seam so `assert_is_read_only_system` can enforce that the combat-facing phase-strip ingest path has no combat-state writers.
  - Kept the runtime regression focused on `CombatState` snapshots plus fake `CombatEvent` messages so the proof remains headless and does not require egui rendering or a display server.
duration: 
verification_result: passed
completed_at: 2026-05-20T20:48:52.973Z
blocker_discovered: false
---

# T03: Added a read-only Bevy seam and regression test proving the windowed phase strip only projects combat beat events into UI-owned state.

**Added a read-only Bevy seam and regression test proving the windowed phase strip only projects combat beat events into UI-owned state.**

## What Happened

Updated `src/ui/phase_strip.rs` to split beat ingestion into a pure latest-beat helper plus a dedicated read-only Bevy system seam, then kept `observe_combat_beats` as the UI-owned projector that mutates only `PhaseStripDisplay`. Added `tests/phase_strip_readonly.rs` under the windowed feature to assert the seam is a Bevy read-only system, feed fake `OnCombatBeat` messages through a minimal `App`, confirm the latest beat wins, and prove `CombatState` remains unchanged across beat, non-beat, and empty-event updates.

## Verification

Verified the dedicated windowed regression with `cargo test --test phase_strip_readonly --features windowed`, then ran the full headless `cargo test` suite and `cargo build --features windowed`. The targeted regression passed all three cases, the full test suite passed, and the windowed build completed successfully.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test phase_strip_readonly --features windowed` | 0 | ✅ pass | 5369ms |
| 2 | `cargo test` | 0 | ✅ pass | 7545ms |
| 3 | `cargo build --features windowed` | 0 | ✅ pass | 4757ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/phase_strip.rs`
- `tests/phase_strip_readonly.rs`
