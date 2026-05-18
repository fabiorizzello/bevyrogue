# S02: Clip typed schema + loader + lossless conversion

**Goal:** Schema tipizzato Clip + RonAssetPlugin::<Clip> loader, Agumon clip.ron generato lossless da agumon_atlas.json.
**Demo:** cargo test carica Agumon clip.ron come Clip tipizzato e asserisce equivalenza geometrica lossless con agumon_atlas.json (frame_size/columns/rows/total_frames + ogni range clip).

## Must-Haves

- Clip type + loader (S02) delivers typed asset; Agumon clip.ron matches atlas json geometry.

## Proof Level

- This slice proves: Contract

## Integration Closure

Clip asset registered in DataPlugin; lossless data consistency verified.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [ ] **T01: Define Clip types** `est:1h`
  Define Clip and related types (ClipRange, Meta) based on draft 02-02. Use a dedicated module src/combat/blueprints/anim_graph/clip.rs (or similar).
  - Files: `src/combat/blueprints/anim_graph/clip.rs`
  - Verify: cargo check

- [ ] **T02: Register Clip asset and update DataPlugin** `est:1h`
  Register Clip asset and update DataPlugin to load it.
  - Files: `src/data/mod.rs`
  - Verify: cargo check

- [ ] **T03: Generate Agumon clip.ron from atlas json** `est:1h`
  Manually generate Agumon clip.ron from agumon_atlas.json, preserving exact geometry. Place it in assets/digimon/agumon/clip.ron.
  - Files: `assets/digimon/agumon/clip.ron`
  - Verify: ls assets/digimon/agumon/clip.ron

- [ ] **T04: Verify clip geometry parity** `est:1h`
  Add a contract test to verify that Agumon's clip.ron matches the geometry in agumon_atlas.json.
  - Files: `tests/clip_geometry_parity.rs`
  - Verify: cargo test --test clip_geometry_parity

## Files Likely Touched

- src/combat/blueprints/anim_graph/clip.rs
- src/data/mod.rs
- assets/digimon/agumon/clip.ron
- tests/clip_geometry_parity.rs
