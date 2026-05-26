---
id: T01
parent: S04
milestone: M006
key_files:
  - src/windowed/mod.rs
  - src/windowed/digimon/mod.rs
  - src/windowed/digimon/agumon/mod.rs
key_decisions:
  - `UiPlugin` retains ownership of `CueRegistry` initialization; per-Digimon modules only populate shared registries after engine setup.
  - Windowed per-Digimon presentation registration now flows through `crate::windowed::digimon::register_all(app)` -> `agumon::register(app)` instead of inline Agumon startup wiring in `src/windowed/mod.rs`.
duration: 
verification_result: passed
completed_at: 2026-05-26T13:43:03.044Z
blocker_discovered: false
---

# T01: Scaffolded the windowed per-Digimon registration seam and moved Agumon cue registration into src/windowed/digimon/agumon/mod.rs.

**Scaffolded the windowed per-Digimon registration seam and moved Agumon cue registration into src/windowed/digimon/agumon/mod.rs.**

## What Happened

Created the internal `src/windowed/digimon/` module seam with `register_all(app)` delegating to `agumon::register(app)`, added `mod digimon;` to `src/windowed/mod.rs`, and wired `UiPlugin::build` to call `crate::windowed::digimon::register_all(app)` exactly once after `CueRegistry` initialization. The Agumon cue-registration startup system now lives in `src/windowed/digimon/agumon/mod.rs`, while `UiPlugin` continues to own shared resource initialization and the engine file no longer defines `register_agumon_cues`. During this repair pass I also removed the stray `.gsd/milestones/M006/slices/S04/S04-SUMMARY.md` artifact because the slice is still pending in the DB and that file was causing the completion gate to fail despite T01 itself being correctly implemented.

## Verification

Verified the T01 seam with three fresh checks: `RUSTFLAGS='-D warnings' cargo build --features windowed` succeeded with zero warnings, `cargo test --features windowed --test windowed_only` passed all 62 tests, and a targeted `rg` seam check confirmed `register_agumon_cues` is absent from `src/windowed/mod.rs` while `register_all(app)` and the moved Agumon registration functions exist under `src/windowed/digimon/`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS='-D warnings' cargo build --features windowed` | 0 | ✅ pass | 744ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | ✅ pass | 1466ms |
| 3 | `rg seam checks for register_agumon_cues/register_all/register` | 0 | ✅ pass | 14ms |

## Deviations

Removed the premature `.gsd/milestones/M006/slices/S04/S04-SUMMARY.md` artifact so the still-pending slice no longer falsely advertises completion. No code-path deviation from the T01 plan was needed.

## Known Issues

Remaining S04 tasks are still pending in the GSD database; the slice must not be completed again until those task records are reconciled and all four tasks are marked complete.

## Files Created/Modified

- `src/windowed/mod.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
