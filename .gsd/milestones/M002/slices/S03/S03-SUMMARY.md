---
id: S03
parent: M002
milestone: M002
provides:
  - A top-center windowed phase strip driven exclusively by `CombatEventKind::OnCombatBeat` messages.
  - A UI-owned `PhaseStripDisplay` resource and pure label helpers that can be inspected without a display server.
  - Structural regression proof that the phase-strip ingress never mutates `CombatState`.
requires:
  []
affects:
  []
key_files:
  - src/ui/phase_strip.rs
  - src/ui/mod.rs
  - src/windowed/mod.rs
  - tests/phase_strip_readonly.rs
key_decisions:
  - Store only the last observed `CombatBeatId` in `PhaseStripDisplay` so the phase strip remains UI-owned instead of mirroring combat state.
  - Project combat events into the phase strip by chaining observe-then-render in `EguiPrimaryContextPass`, so same-frame egui rendering sees the latest beat.
  - Prove the combat-facing seam structurally with `assert_is_read_only_system` and fake `CombatEvent` messages instead of relying on manual review.
patterns_established:
  - Project event-stream UI state into a windowed-only resource rather than reading or mutating gameplay state directly.
  - Chain message ingestion before egui rendering inside `EguiPrimaryContextPass` for same-frame UI coherence.
  - Keep windowed UI seams headless-safe with feature gates and focused regression tests.
observability_surfaces:
  - `PhaseStripDisplay` exposes the last observed beat for inspection.
  - `tests/phase_strip_readonly.rs` covers no-event, non-beat, and latest-beat-wins cases.
  - Fresh `cargo test` / windowed test / windowed build evidence records the slice closeout status.
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-20T21:02:24.505Z
blocker_discovered: false
---

# S03: Section 9 phase strip live (event-driven)

**Delivered a windowed, event-driven combat phase strip that projects `CombatEvent::OnCombatBeat` into a UI-owned egui banner without mutating combat state.**

## What Happened

T01 established the windowed-only `PhaseStripDisplay` model plus pure beat-to-label helpers in `src/ui/phase_strip.rs` and exported the module from `src/ui/mod.rs`, making the display contract inspectable without a display server. T02 wired the live windowed path: `observe_combat_beats` reads `MessageReader<CombatEvent>`, filters `CombatEventKind::OnCombatBeat { beat }`, updates only `PhaseStripDisplay`, and `render_phase_strip` draws the compact top-center egui strip; `UiPlugin::build()` initializes the display resource and chains observe-then-render inside `EguiPrimaryContextPass` in `src/windowed/mod.rs`. T03 added `tests/phase_strip_readonly.rs` plus the read-only `read_latest_observed_combat_beat` seam to prove the UI ingress is structurally read-only, the latest beat wins, and empty or non-beat updates leave `CombatState` untouched. T04 re-ran the slice closeout gates so the headless default build, the windowed structural regression, and the real windowed compile path all stay green.

## Verification

Fresh closeout verification passed with `cargo test`, `cargo test --test phase_strip_readonly --features windowed`, and `cargo build --features windowed`. These checks re-proved the headless-first contract, the negative/read-only event-reader cases, and the actual `UiPlugin` windowed wiring.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

Health signal: `PhaseStripDisplay.current_beat` and the focused windowed regression indicate whether the UI is consuming `OnCombatBeat` messages. Failure signal: the strip stays empty/stale while combat proceeds, or the `phase_strip_readonly` regression/build gates fail. Recovery procedure: rerun `cargo test --test phase_strip_readonly --features windowed` and `cargo build --features windowed`, then inspect `src/ui/phase_strip.rs` and `src/windowed/mod.rs` for message-reader wiring drift. Monitoring gaps: there is no runtime telemetry beyond the inspectable UI-owned resource and regression coverage.

## Deviations

None.

## Known Limitations

The slice is proven by automated tests and compile verification rather than a live desktop smoke run; visual polish and environment-specific display issues remain outside this contract.

## Follow-ups

Optional future slices can add richer reactive labels or live visual smoke coverage, but S03 itself is complete with the current event-driven/read-only contract.

## Files Created/Modified

- `src/ui/phase_strip.rs` — Defines the UI-owned phase-strip display model, event-reader ingest path, egui banner renderer, and focused unit tests.
- `src/ui/mod.rs` — Exports the phase-strip module only under the `windowed` feature gate.
- `src/windowed/mod.rs` — Initializes `PhaseStripDisplay` and schedules phase-strip observe/render inside `EguiPrimaryContextPass`.
- `tests/phase_strip_readonly.rs` — Adds the read-only structural regression and negative-case coverage for the windowed phase-strip seam.
