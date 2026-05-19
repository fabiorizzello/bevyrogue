# S03: S03: Cross-asset validation with adapter-based catalogs

**Goal:** Cross-asset validation using adapter-provided catalogs, not direct animation-core coupling.
**Demo:** Valid graph+clip assets pass required checks; broken fixtures fail with typed diagnostics; cross-asset checks use adapter-provided catalogs.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: S03 implementation** `est:4h`
  Implement S03: Cross-asset validation with adapter-based catalogs
  - Files: `src/animation/mod.rs`
  - Verify: cargo test

## Files Likely Touched

- src/animation/mod.rs
