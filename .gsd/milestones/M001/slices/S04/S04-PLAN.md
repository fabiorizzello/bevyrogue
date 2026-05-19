# S04: S04: Non-Agumon generic path + hot-reload proof

**Goal:** Generic animation pipeline validated with non-Agumon data and windowed hot-reload proof.
**Demo:** Non-Agumon animation assets validate through the same generic path, and cargo run --features windowed proves manual hot reload without crash or corrupted world state.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: S04 implementation** `est:4h`
  Implement S04: Non-Agumon generic path + hot-reload proof
  - Files: `src/animation/mod.rs`
  - Verify: cargo test

## Files Likely Touched

- src/animation/mod.rs
