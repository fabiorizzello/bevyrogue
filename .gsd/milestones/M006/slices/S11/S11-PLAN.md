# S11: Data-driven catalog discovery replacing DEFAULT_ANIM_GRAPH/CLIP/STANCE_PATHS

**Goal:** Replace the hardcoded DEFAULT_ANIM_GRAPH/CLIP/STANCE path constants (src/animation/plugin.rs) with data-driven catalog discovery, so adding a Digimon is a matter of dropping its asset folder rather than editing path constants in the lib layer.
**Demo:** New Digimon discovered from data without editing path constants; headless catalog test

## Must-Haves

- A new Digimon's anim_graph/clip/stance assets are discovered from the assets/data + assets/digimon layout without editing any DEFAULT_*_PATHS constant; headless catalog test proves discovery of the existing roster and that an added entry is picked up. Determinism preserved (sorted discovery, no wall-clock).

## Proof Level

- This slice proves: headless catalog test

## Verification

- Catalog discovery logs (once) the roster it resolved and warns on a digimon folder missing a required asset, so a misfiled Digimon is diagnosable.

## Tasks

- [ ] **T01: Define the catalog discovery source of truth** `est:M`
  Introduce a data-driven catalog that enumerates Digimon presentation assets (anim_graph, clip, stance) from the on-disk/asset layout instead of the DEFAULT_*_PATHS constants. Deterministic ordering, no wall-clock, no unseeded RNG (R004).
  - Files: `src/animation/plugin.rs`, `src/animation/registry.rs`
  - Verify: cargo test (headless green); cargo check

- [ ] **T02: Cut over loaders to the catalog and remove the constants** `est:M`
  Repoint asset loading to consume the catalog and delete DEFAULT_ANIM_GRAPH/CLIP/STANCE_PATHS. Ensure the existing roster loads identically. Warn-once on a folder missing a required asset.
  - Files: `src/animation/plugin.rs`
  - Verify: cargo test (headless green); cargo test --features windowed --test windowed_only (green)

- [ ] **T03: Catalog discovery headless test** `est:S`
  Add a headless test asserting the catalog discovers the current roster and that adding a fixture entry is picked up without code edits, locking the data-driven contract.
  - Files: `tests/assets_data/catalog_discovery.rs`
  - Verify: cargo test --test assets_data (catalog discovery green)

## Files Likely Touched

- src/animation/plugin.rs
- src/animation/registry.rs
- tests/assets_data/catalog_discovery.rs
