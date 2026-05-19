---
id: T01
parent: S01
milestone: M002
provides:
  - AnimGraphId required newtype field on AnimGraph
  - FrameCue/FrameCueCommand/ReleaseKernelCue schema types
  - Predicate::KernelCue variant
  - anim_graph_parse / anim_graph_asset / anim_validation integration tests
key_files:
  - src/animation/anim_graph.rs
  - src/animation/validation/predicate.rs
  - src/lib.rs
  - assets/digimon/agumon/anim_graph.ron
  - assets/digimon/renamon/anim_graph.ron
  - assets/test/animation_validation/valid_anim_graph.ron
  - assets/test/animation_validation/broken_anim_graph.ron
  - tests/anim_graph_parse.rs
  - tests/anim_graph_asset.rs
  - tests/anim_validation.rs
  - Cargo.toml
key_decisions:
  - FrameCueCommand is a closed enum (Presentation | ReleaseKernel) — no escape hatch variant
  - ReleaseKernelCue is a unit struct so RON serializes it as ()
  - KernelCue added to Predicate as a unit variant (no payload needed)
patterns_established:
  - AnimGraphId follows the same transparent-newtype pattern as ClipId/NodeId/etc.
  - cues field on AnimNode uses #[serde(default)] so existing RON assets load without changes beyond the id field
observability_surfaces:
  - none
duration: ~15 min
verification_result: passed
completed_at: 2026-05-19
blocker_discovered: false
---

# T01: Closed-enum schema extensions + atomic id/asset/test migration

**Added `AnimGraphId`, `FrameCue`/`FrameCueCommand`/`ReleaseKernelCue`, and `Predicate::KernelCue` to the closed schema; migrated all RON assets and tests atomically — 12/12 tests green.**

## What Happened

Extended `src/animation/anim_graph.rs` with five schema additions:
1. `AnimGraphId` — transparent newtype, mirrors the ClipId/NodeId pattern. Added as `id: AnimGraphId` (required) to `AnimGraph`.
2. `FrameCue { at: u32, command: FrameCueCommand }` — frame-indexed command carrier on `AnimNode.cues` (defaulting to empty vec via `#[serde(default)]`).
3. `FrameCueCommand` — closed enum with two variants: `Presentation(Command)` (delegates to existing Command vocabulary) and `ReleaseKernel(ReleaseKernelCue)` (new kernel release signal).
4. `ReleaseKernelCue` — unit struct; serializes as `()` in RON.
5. `Predicate::KernelCue` — unit variant added to the closed Predicate enum; fires when the runtime sees a ReleaseKernelCue at the current frame.

Updated `src/animation/validation/predicate.rs` to add `Predicate::KernelCue` to the exhaustive match (no validation logic needed — it is self-contained).

Exported `pub mod animation;` from `src/lib.rs` so integration tests can reach `bevyrogue::animation::*`.

Patched all four RON asset files to include the now-required `id` field. Registered three new `[[test]]` entries in `Cargo.toml`.

Created three integration test files covering: schema round-trips, real-asset parsing, and validation logic with the new types.

## Verification

`cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation`

Full `cargo test` (all suites) also green — no regressions.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation` | 0 | pass | ~10s compile + <1s run |
| 2 | `cargo test` (full suite) | 0 | pass | ~15s |

anim_graph_parse: 7 tests (id_required, cues_absent, release_kernel_cue, presentation_cue, unknown_variant_rejected, unknown_field_rejected, kernel_cue_predicate)
anim_graph_asset: 2 tests (agumon, renamon)
anim_validation: 3 tests (valid_passes, broken_has_error, kernel_cue_validates_clean)

## Diagnostics

Schema types live in `src/animation/anim_graph.rs`. RON assets are under `assets/`. Integration tests in `tests/anim_graph_*.rs` and `tests/anim_validation.rs`.

## Deviations

None. The implementation follows the task plan exactly.

## Known Issues

None.

## Files Created/Modified

- `src/animation/anim_graph.rs` — added AnimGraphId, id field on AnimGraph, FrameCue/FrameCueCommand/ReleaseKernelCue, Predicate::KernelCue
- `src/animation/validation/predicate.rs` — added KernelCue arm to exhaustive match
- `src/lib.rs` — added `pub mod animation`
- `assets/digimon/agumon/anim_graph.ron` — added `id: "agumon_skill"`
- `assets/digimon/renamon/anim_graph.ron` — added `id: "renamon_skill"`
- `assets/test/animation_validation/valid_anim_graph.ron` — added `id: "test_valid"`
- `assets/test/animation_validation/broken_anim_graph.ron` — added `id: "test_broken"`
- `tests/anim_graph_parse.rs` — new: 7 serde round-trip tests
- `tests/anim_graph_asset.rs` — new: 2 real-asset parse tests
- `tests/anim_validation.rs` — new: 3 validation logic tests
- `Cargo.toml` — registered 3 new [[test]] targets
