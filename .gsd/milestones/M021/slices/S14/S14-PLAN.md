# S14: Prove two-clock parity and extension boundaries

**Goal:** Prove the remaining milestone boundary and parity claims that were left implicit after the main migration slices, with explicit evidence for extension isolation and cross-clock equivalence.
**Demo:** After this: HeadlessAuto and Windowed produce equivalent intent streams; blueprint modules stay bevy-free; add-new-digimon isolation and single-register boundaries are evidenced.

## Must-Haves

- Fresh proof covers `HeadlessAuto` and `Windowed` intent-stream equivalence.
- Fresh proof covers `rg "use bevy" src/combat/blueprints/` → 0.
- Fresh proof covers the 'add new digimon touches only blueprint and data dirs' milestone claim or truthfully narrows it.
- Slice artifacts explain the cross-slice producer/consumer boundaries that validation could not infer from the roadmap alone.

## Proof Level

- This slice proves: Targeted integration proofs plus structural grep and fixture evidence on the live tree.

## Integration Closure

This slice closes the validation gap between isolated migration slices by proving that the same generic framework composes across clocks, blueprint boundaries, and the add-a-new-digimon extension workflow.

## Verification

- Adds explicit parity and boundary evidence so two-clock behavior, blueprint isolation, and extension seams are visible from current tests and UAT artifacts.

## Tasks

- [x] **T01: Prove HeadlessAuto and Windowed intent-stream parity** `est:0.75d`
  Add or tighten an explicit two-clock parity test that drives the same encounter or compiled timeline under HeadlessAuto and Windowed semantics, then asserts identical emitted intent streams and end-of-cast outcomes.
  - Files: `src/combat/api/clock.rs`, `src/combat/api/runner.rs`, `src/windowed.rs`, `src/headless.rs`, `tests`
  - Verify: cargo test -- --nocapture windowed || true
cargo test -- --nocapture parity || true

- [x] **T02: Prove blueprint isolation and no-Bevy shared boundaries** `est:0.5d`
  Audit blueprint modules for forbidden Bevy imports and the one-module one-register contract. Add a focused structural proof or test fixture that shows blueprint integration remains isolated to owner modules and shared registries.
  - Files: `src/combat/blueprints`, `tests`
  - Verify: rg "use bevy" src/combat/blueprints/
rg -n "fn register\(" src/combat/blueprints/

- [ ] **T03: Prove add-new-digimon isolation and capture cross-slice boundaries** `est:0.75d`
  Create or update a scripted add-new-digimon proof that demonstrates the intended extension boundary and records the cross-slice integration contracts that consume the earlier migration work. If the current code still needs shared edits, capture the smallest truthful scope and document the remaining gap explicitly.
  - Files: `src/combat/blueprints`, `src/data`, `tests`
  - Verify: cargo test -- --nocapture add_new_digimon || true
cargo test -- --nocapture blueprint || true

## Files Likely Touched

- src/combat/api/clock.rs
- src/combat/api/runner.rs
- src/windowed.rs
- src/headless.rs
- tests
- src/combat/blueprints
- src/data
