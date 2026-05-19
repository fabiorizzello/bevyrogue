# S04: Roster ready assets and real hot reload proof

**Goal:** Author non-Agumon animation assets (Renamon), implement dynamic roster discovery and real-data catalog synchronization, and prove hot-reload stability in the windowed environment.
**Demo:** Non-Agumon animation assets validate through the same generic path, and `cargo run --features windowed` proves manual hot reload without crash or corrupted world state.

## Must-Haves

- Renamon clip and graph assets authored and validated.
- Validation catalogs automatically synchronized with real SkillBook and StatusEffectKind data.
- AnimationAssetPlugin loads and tracks the full roster of animation assets.
- Manual hot-reload proof documented and successful in the windowed environment.

## Proof Level

- This slice proves: operational

## Verification

- Provides a visual validation status indicator in the windowed UI and logs typed diagnostics for the entire roster.

## Tasks

- [x] **T01: Author Renamon animation assets** `est:45m`
  Author Renamon's animation assets to prove the generic roster-ready path (R007).
  - Files: `assets/digimon/renamon/clip.ron`, `assets/digimon/renamon/anim_graph.ron`
  - Verify: ls assets/digimon/renamon/clip.ron assets/digimon/renamon/anim_graph.ron

- [x] **T02: Implement Catalog Sync and Dynamic Discovery** `est:1h`
  Implement dynamic discovery and catalog synchronization in the animation plugin to move beyond Agumon-only hardcoding (R003, R007).
  - Files: `src/animation/plugin.rs`
  - Verify: cargo test --test anim_asset_validation

- [x] **T03: Visual Validation Status and Hot-Reload Proof** `est:45m`
  Add a visual validation status to the windowed UI and perform the manual hot-reload proof required by R006.
  - Files: `src/windowed.rs`
  - Verify: grep -q "AnimationValidationState" src/windowed.rs

## Files Likely Touched

- assets/digimon/renamon/clip.ron
- assets/digimon/renamon/anim_graph.ron
- src/animation/plugin.rs
- src/windowed.rs
