# S10: Split render into playback/spawn/effects/feedback/clock submodules

**Goal:** Split the remaining src/windowed/render.rs into focused submodules (playback, spawn, effects, feedback, clock) and break up the oversized advance_digimon_presentation function into per-concern steps. Adjust the source-contract tests to the new module shape. Behavior-preserving decomposition.
**Demo:** render decomposed; advance_digimon_presentation broken up; source-contract tests adjusted; windowed tests green

## Must-Haves

- render becomes a thin orchestrator delegating to playback/spawn/effects/feedback/clock submodules; advance_digimon_presentation is decomposed into named per-concern functions; source-contract tests updated to assert the new boundaries; windowed_only and headless suites green; warnings-clean windowed build.

## Proof Level

- This slice proves: full test suite green + adjusted source-contract tests (refactor parity)

## Verification

- Keep the AnimationClock catch-up cap and barrier-release-on-tick behavior intact (per existing windowed clock semantics); preserve warn-once diagnostics across the split.

## Tasks

- [x] **T01: Split render into per-concern submodules** `est:L`
  Carve render.rs into src/windowed/render/{playback,spawn,effects,feedback,clock}.rs, each owning one concern, with render/mod.rs as the orchestrator. Move systems verbatim where possible; no behavior change to clock catch-up or barrier release.
  - Files: `src/windowed/render.rs`, `src/windowed/render/mod.rs`, `src/windowed/render/playback.rs`, `src/windowed/render/spawn.rs`, `src/windowed/render/effects.rs`, `src/windowed/render/feedback.rs`, `src/windowed/render/clock.rs`
  - Verify: RUSTFLAGS='-D warnings' cargo build --features windowed (clean); cargo test --features windowed --test windowed_only (green)

- [x] **T02: Break up advance_digimon_presentation** `est:M`
  Decompose the advance_digimon_presentation function into named per-concern steps (e.g. tick clock, advance playback, release barriers, drive feedback) so each is independently readable and testable. Behavior identical.
  - Files: `src/windowed/render/playback.rs`, `src/windowed/render/clock.rs`
  - Verify: cargo test --features windowed --test windowed_only (green); cargo test (headless green)

- [ ] **T03: Adjust source-contract tests to new module shape** `est:M`
  Update the windowed source-contract tests (renamon_extension_contract.rs, agumon_module_extraction.rs) to assert the new module boundaries (engine-generic render core vs per-species data) rather than the old single-file layout. Tests must still enforce the zero-engine-edit contract.
  - Files: `tests/windowed_only/renamon_extension_contract.rs`, `tests/windowed_only/agumon_module_extraction.rs`
  - Verify: cargo test --features windowed --test windowed_only (green, contracts re-pointed)

## Files Likely Touched

- src/windowed/render.rs
- src/windowed/render/mod.rs
- src/windowed/render/playback.rs
- src/windowed/render/spawn.rs
- src/windowed/render/effects.rs
- src/windowed/render/feedback.rs
- src/windowed/render/clock.rs
- tests/windowed_only/renamon_extension_contract.rs
- tests/windowed_only/agumon_module_extraction.rs
