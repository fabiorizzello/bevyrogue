---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Verify parse-ability in a contract test

Add a contract test to verify that Agumon's anim_graph.ron can be parsed into the typed AnimGraph struct.

## Inputs

- `assets/digimon/agumon/anim_graph.ron`

## Expected Output

- `tests/anim_graph_parse.rs`

## Verification

cargo test --test anim_graph_parse
