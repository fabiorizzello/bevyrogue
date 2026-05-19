# S01: S01: Typed AnimGraph asset loader

**Goal:** Typed AnimGraph asset loader with boot-time validation against a closed schema.
**Demo:** cargo test loads an Agumon anim_graph.ron as a typed asset through the new animation module and rejects out-of-vocabulary schema values with typed errors.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: S01 implementation** `est:4h`
  Implement S01: Typed AnimGraph asset loader
  - Files: `src/animation/mod.rs`
  - Verify: cargo test

## Files Likely Touched

- src/animation/mod.rs
