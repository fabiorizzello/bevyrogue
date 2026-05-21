# S03: S03

**Goal:** Make the Section 9 phase strip live in the windowed UI by deriving its display state from CombatEvent::OnCombatBeat messages only, while proving the phase-strip UI path does not mutate combat state.
**Demo:** §9 phase strip updates from EventReader<CombatEvent>; a structural test asserts the UI path never mutates combat state.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Phase-strip display model and pure CombatBeatId→label contract added**
  - Files: `src/ui/phase_strip.rs`, `src/ui/mod.rs`
  - Verify: cargo test --features windowed phase_strip

- [x] **T02: Wire EventReader-driven egui phase strip into UiPlugin**
  - Files: `src/ui/phase_strip.rs`, `src/windowed/mod.rs`
  - Verify: cargo build --features windowed

- [x] **T03: Prove phase-strip UI path is combat-read-only**
  - Files: `tests/phase_strip_readonly.rs`, `src/ui/phase_strip.rs`
  - Verify: cargo test --test phase_strip_readonly --features windowed

- [x] **T04: Close S03 verification across headless and windowed gates**
  - Verify: cargo test

## Files Likely Touched

- src/ui/phase_strip.rs
- src/ui/mod.rs
- src/windowed/mod.rs
- tests/phase_strip_readonly.rs
