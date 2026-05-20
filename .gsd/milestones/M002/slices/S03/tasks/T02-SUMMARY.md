---
id: T02
parent: S03
milestone: M002
key_files:
  - src/ui/phase_strip.rs
  - src/windowed/mod.rs
  - tests/phase_strip_readonly.rs
key_decisions:
  - Kept the phase strip as a pure projection of `CombatEventKind::OnCombatBeat` by storing only the last observed beat in `PhaseStripDisplay`, not any combat-state mirror.
  - Scheduled phase-strip observe/render as a chained `EguiPrimaryContextPass` pair so the latest beat from the reader window is applied before the same-frame egui banner renders.
duration: 
verification_result: passed
completed_at: 2026-05-20T20:59:03.322Z
blocker_discovered: false
---

# T02: Wired the windowed egui phase strip into UiPlugin so it projects `CombatEvent::OnCombatBeat` into a UI-owned top-center banner.

**Wired the windowed egui phase strip into UiPlugin so it projects `CombatEvent::OnCombatBeat` into a UI-owned top-center banner.**

## What Happened

Verified the existing T02 implementation in `src/ui/phase_strip.rs` and `src/windowed/mod.rs`. The phase-strip module already ingests combat messages through `MessageReader<CombatEvent>`, filters `CombatEventKind::OnCombatBeat { beat }`, updates only the UI-owned `PhaseStripDisplay`, and renders a compact top-center egui strip from the derived label/state. `UiPlugin::build()` already initializes `PhaseStripDisplay` and chains `observe_combat_beats` before `render_phase_strip` inside `EguiPrimaryContextPass`, keeping the windowed path event-driven while preserving the headless-first feature gates. The negative/read-only regression coverage remains in `tests/phase_strip_readonly.rs`, which proves empty updates and non-beat events do not disturb combat state.

## Verification

Ran fresh verification for the T02 contract and slice-level gating: `cargo test` passed in the default headless build, `cargo test --test phase_strip_readonly --features windowed` passed the read-only and negative regression cases, and `cargo build --features windowed` confirmed the real windowed egui/plugin wiring compiles.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | ✅ pass | 3308ms |
| 2 | `cargo test --test phase_strip_readonly --features windowed` | 0 | ✅ pass | 358ms |
| 3 | `cargo build --features windowed` | 0 | ✅ pass | 218ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/phase_strip.rs`
- `src/windowed/mod.rs`
- `tests/phase_strip_readonly.rs`
