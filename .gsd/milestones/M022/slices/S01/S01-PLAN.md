# S01: AnimGraph typed schema + loader

**Goal:** Schema tipizzato AnimGraph (nodi, edges, Predicate §D chiuso, vocabolario Command §C+§C2 chiuso, ParamRef §S, TargetShape §C3) + RonAssetPlugin::<AnimGraph> loader; Agumon anim_graph.ron baby_flame (forma §M).
**Demo:** cargo test carica Agumon anim_graph.ron baby_flame (forma §M) come AnimGraph tipizzato; asserzioni su nodi, edge, Predicate §D, Command §C/§C2, ParamRef §S; un RON con vocabolo fuori-enum è rifiutato come DataError tipizzato.

## Must-Haves

- AnimGraph type + loader (S01) delivers typed asset from real Agumon RON.

## Proof Level

- This slice proves: Contract

## Integration Closure

AnimGraph asset registered in DataPlugin; LoadedWithDependencies tracker integration.

## Verification

- None (loader-only)

## Tasks

- [ ] **T01: Define AnimGraph and related types** `est:2h`
  Define all AnimGraph related types (Node, Edge, Command, Predicate, etc.) based on draft 02-02b. Use a dedicated module src/combat/blueprints/anim_graph/types.rs.
  - Files: `src/combat/blueprints/anim_graph/types.rs`
  - Verify: cargo check

- [ ] **T02: Register AnimGraph asset and update DataPlugin** `est:1h`
  Register AnimGraph with RonAssetPlugin and update DataPlugin to load it. Add handles and trackers in src/data/mod.rs.
  - Files: `src/data/mod.rs`, `src/data/anim_graph_ron.rs`
  - Verify: cargo check

- [ ] **T03: Create Agumon baby_flame anim_graph.ron** `est:1h`
  Create the initial anim_graph.ron for Agumon (baby_flame) in assets/digimon/agumon/anim_graph.ron. Create the directory if needed.
  - Files: `assets/digimon/agumon/anim_graph.ron`
  - Verify: ls assets/digimon/agumon/anim_graph.ron

- [ ] **T04: Verify parse-ability in a contract test** `est:1h`
  Add a contract test to verify that Agumon's anim_graph.ron can be parsed into the typed AnimGraph struct.
  - Files: `tests/anim_graph_parse.rs`
  - Verify: cargo test --test anim_graph_parse

## Files Likely Touched

- src/combat/blueprints/anim_graph/types.rs
- src/data/mod.rs
- src/data/anim_graph_ron.rs
- assets/digimon/agumon/anim_graph.ron
- tests/anim_graph_parse.rs
