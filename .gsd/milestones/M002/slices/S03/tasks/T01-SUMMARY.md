---
id: T01
parent: S03
milestone: M002
key_files:
  - src/ui/phase_strip.rs
  - src/ui/mod.rs
key_decisions:
  - Kept the phase-strip contract windowed-only and UI-owned by storing only an optional `CombatBeatId` plus pure label helpers; `Damage` maps to the `Impact` display label and `ExtraHit` maps to `Chain` explicitly.
duration: 
verification_result: passed
completed_at: 2026-05-20T20:38:40.764Z
blocker_discovered: false
---

# T01: Added a windowed-only phase-strip display model with pure beat-to-label mapping tests.

**Added a windowed-only phase-strip display model with pure beat-to-label mapping tests.**

## What Happened

Added a new `src/ui/phase_strip.rs` module gated behind the `windowed` feature and exported it from `src/ui/mod.rs`. The module defines a presentation-only `PhaseStripDisplay` resource that tracks only the last observed `CombatBeatId`, plus pure helpers that map combat beats into stable Section 9 display phases and labels. The mapping stays explicit for non-core beats by folding `Damage` into the `Impact` display phase and presenting `ExtraHit` as `Chain`. Added focused unit tests covering the empty/default state, stable non-empty labels for every `CombatBeatId::ALL` variant, canonical labels for the core cycle, and the explicit grouped mapping for `Damage`/`ExtraHit`, proving the contract can be verified without loading the windowed app or touching gameplay state.

## Verification

Verified the new contract with `cargo test --features windowed phase_strip`, which ran and passed the five `ui::phase_strip` tests for empty-state behavior, full beat coverage, canonical core labels, and explicit grouped mapping. Verified the new module stays hidden from headless builds with `cargo check` (default features). Also ran `rustfmt --check src/ui/phase_strip.rs` to confirm the new file matches local Rust formatting.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `rustfmt --check src/ui/phase_strip.rs` | 0 | ✅ pass | 20ms |
| 2 | `cargo test --features windowed phase_strip` | 0 | ✅ pass | 45134ms |
| 3 | `cargo check` | 0 | ✅ pass | 310ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/ui/phase_strip.rs`
- `src/ui/mod.rs`
