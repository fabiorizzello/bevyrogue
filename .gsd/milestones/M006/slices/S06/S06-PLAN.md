# S06: Graph registry processes all matching graph events so Renamon sprite spawns

**Goal:** Fix populate_graph_registries so every queued AnimationGraph asset event is processed in a batch, not just the first. This is the root cause of Renamon's sprite never spawning: the early return inside the event loop (src/animation/registry.rs:276,279) starves all graphs after the first one to load.
**Demo:** Headless registry test proves every queued graph asset populates; windowed run shows Renamon idle sprite present

## Must-Haves

- Headless test proves that when multiple graph asset events are queued in one batch, all corresponding registries populate (not just one). Windowed run (manual, K001) shows Renamon's idle sprite present alongside Agumon's.

## Proof Level

- This slice proves: headless test + manual windowed sign-off (K001)

## Verification

- Add a warn-once when a graph asset event arrives but no registry entry can be built for it, so a future missing-graph regression is visible in logs rather than silent.

## Tasks

- [x] **T01: Reproduce the single-graph-per-batch starvation in a headless test** `est:S`
  Write a failing headless test that queues two AnimationGraph asset-load events in a single update and asserts both registries populate. Confirm it fails against the current early-return behavior at registry.rs lines 276 and 279.
  - Files: `tests/animation/case.rs`, `src/animation/registry.rs`
  - Verify: cargo test --test animation -- registry (new case fails red, reproducing the bug)

- [x] **T02: Process all matching graph events per batch** `est:S`
  Replace the early return statements in the asset-event loop with continue (or restructure so each event is handled independently) so every AnimationGraph load in a batch populates its registry. Keep per-event error isolation: a bad graph warns and is skipped without aborting the loop.
  - Files: `src/animation/registry.rs`
  - Verify: cargo test --test animation (T01 case now green); cargo test (full headless suite green)

- [x] **T03: Warn-once on unbuildable graph event** `est:S`
  Emit a single warn log (deduplicated per graph handle) when a graph asset event cannot produce a registry entry, giving the handle/path for diagnosis. No secrets, no per-frame spam.
  - Files: `src/animation/registry.rs`
  - Verify: cargo test (headless green); manual: cargo winx shows no warn for Renamon/Agumon happy path

## Files Likely Touched

- tests/animation/case.rs
- src/animation/registry.rs
