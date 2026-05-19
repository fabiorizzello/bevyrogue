# S03: Validator L with adapter based checks

**Goal:** Add a generic headless animation validator that joins typed AnimGraph and Clip assets with adapter-provided catalogs, accepts the real Agumon graph+clip path, rejects broken cross-asset references with typed diagnostics, and preserves the src/animation boundary without importing project data internals.
**Demo:** Valid graph+clip assets pass required checks; broken fixtures fail with typed diagnostics; cross-asset checks use adapter-provided catalogs.

## Must-Haves

- Valid real Agumon `anim_graph.ron` + `clip.ron` validates through `src/animation` using catalogs built outside animation core.
- Broken graph/clip fixtures fail with typed, inspectable diagnostics covering missing clip ranges, missing nodes, bad frame spans, missing catalog entries, and malformed graph links.
- Asset-load integration exposes validation state/failure diagnostics in headless Bevy tests and does not depend on `windowed`, winit, wgpu, UI, or human interaction.
- `src/animation` remains generic: no direct imports from `crate::data`, `crate::combat`, or Digimon-specific modules in validator core.

## Proof Level

- This slice proves: Integration contract proof. Real runtime required: no. Human/UAT required: no. Verification is entirely headless through Rust integration tests plus full `cargo test`.

## Integration Closure

Consumes S01 `AnimGraph` schema/loader and S02 `Clip` schema/loader. Introduces validator API, typed diagnostics, adapter catalog seam, and Bevy validation state wiring that S04 can reuse for non-Agumon assets and windowed hot-reload proof. S04 still owns roster expansion and manual `cargo run --features windowed` hot-reload evidence.

## Verification

- Adds typed `AnimationValidationDiagnostic`/report surfaces and a headless `AnimationValidationState` readiness/failure resource so future agents can inspect whether graph+clip validation passed, which check failed, and which graph/node/field/catalog value caused the block.

## Tasks

- [x] **T01: Add pure validation contract and typed diagnostics** `est:2h`
  Why: S03's load-bearing seam is a generic validator inside `src/animation` that can reason over typed `AnimGraph` and `Clip` without knowing about Digimon, combat, or data modules. This task should be executed test-first using the expected skills: decompose-into-slices, design-an-interface, grill-me, tdd, write-docs, bevy, rust-best-practices, verify-before-complete.
  - Files: `src/animation/validation.rs`, `src/animation/mod.rs`, `tests/anim_validation.rs`
  - Verify: cargo test --test anim_validation

- [x] **T02: Prove real Agumon validation through an external adapter catalog** `est:1.5h`
  Why: The validator only proves R005 if real project data can feed its catalogs from outside animation core. This task demonstrates the adapter seam without adding `src/data` or `src/combat` imports to `src/animation`. Expected executor skills: design-an-interface, tdd, bevy, rust-best-practices, verify-before-complete.
  - Files: `tests/anim_validation.rs`
  - Verify: cargo test --test anim_validation

- [x] **T03: Wire headless asset validation state into the animation plugin** `est:2.5h`
  Why: R004 says invalid animation assets at boot fail fast with typed diagnostics, not just that a pure function exists. This task composes S01/S02 asset readiness with the S03 validator in a headless Bevy path while keeping catalogs injectable by adapters. Expected executor skills: decompose-into-slices, tdd, bevy, rust-best-practices, verify-before-complete.
  - Files: `src/animation/plugin.rs`, `tests/anim_asset_validation.rs`, `assets/test/animation_validation/valid_anim_graph.ron`, `assets/test/animation_validation/valid_clip.ron`, `assets/test/animation_validation/broken_anim_graph.ron`, `assets/test/animation_validation/broken_clip.ron`
  - Verify: cargo test --test anim_asset_validation

- [x] **T04: Run full validation closeout and protect prior contracts** `est:45m`
  Why: S03 changes a central asset plugin and public animation module exports, so closeout must prove the new validator and prior S01/S02 contracts together. Expected executor skills: verify-before-complete, test, rust-best-practices.
  - Verify: cargo test

## Files Likely Touched

- src/animation/validation.rs
- src/animation/mod.rs
- tests/anim_validation.rs
- src/animation/plugin.rs
- tests/anim_asset_validation.rs
- assets/test/animation_validation/valid_anim_graph.ron
- assets/test/animation_validation/valid_clip.ron
- assets/test/animation_validation/broken_anim_graph.ron
- assets/test/animation_validation/broken_clip.ron
