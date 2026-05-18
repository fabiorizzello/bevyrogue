# S04: Hot-reload and degenerate stub 5 non-Agumon

**Goal:** Hot-reload working per Agumon (demo --features windowed). I 5 Digimon non-Agumon: anim_graph.ron degenerate (1 nodo all-clip, §N).
**Demo:** cargo run --features windowed con Agumon, edit a caldo di clip.ron/anim_graph.ron ricarica l'asset senza crash né stato sporco (UAT).

## Must-Haves

- Hot-reload operational; stub RONs for other 5 Digimon valid.

## Proof Level

- This slice proves: Operational + UAT

## Integration Closure

Stubs deployed to all digimon assets; AssetServer file-watch enabled.

## Verification

- Asset re-loaded logs

## Tasks

- [ ] **T01: Create degenerate stubs for non-Agumon digimon** `est:1h`
  Create degenerate anim_graph.ron and clip.ron for the other 5 Digimon (Gabumon, Dorumon, Renamon, Patamon, Tentomon) in their respective assets/digimon/<name>/ directories.
  - Files: `assets/digimon/gabumon/anim_graph.ron`, `assets/digimon/dorumon/anim_graph.ron`, `assets/digimon/renamon/anim_graph.ron`, `assets/digimon/patamon/anim_graph.ron`, `assets/digimon/tentomon/anim_graph.ron`
  - Verify: cargo test

- [ ] **T02: UAT: Verify hot-reload live** `est:1h`
  Verify hot-reload functionality in --features windowed. Document the UAT result.
  - Verify: manual UAT

## Files Likely Touched

- assets/digimon/gabumon/anim_graph.ron
- assets/digimon/dorumon/anim_graph.ron
- assets/digimon/renamon/anim_graph.ron
- assets/digimon/patamon/anim_graph.ron
- assets/digimon/tentomon/anim_graph.ron
