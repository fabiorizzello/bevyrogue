---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Register Renamon diamond_storm_leaf cue

Add the OnEnterEffect/enoki mapping for the diamond_storm_leaf node authored in renamon/anim_graph.ron into renamon's windowed module register(app), populating only Renamon's own registry entries per the established per-species seam. No edits to render.rs control flow.

## Inputs

- `src/windowed/digimon/renamon/mod.rs`
- `assets/digimon/renamon/anim_graph.ron`
- `src/windowed/digimon/agumon/mod.rs`

## Expected Output

- `Renamon register() maps diamond_storm_leaf to an enoki effect via the shared registry`

## Verification

cargo test --features windowed --test windowed_only (Renamon cue mapping present); manual cargo winx shows Renamon cast particle
