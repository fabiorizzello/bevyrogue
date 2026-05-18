---
estimated_steps: 10
estimated_files: 4
skills_used: []
---

# T02: Wire AnimGraph asset plugin and real Agumon graph

Skills expected in executor frontmatter: bevy, rust-best-practices, rust-testing, tdd, verify-before-complete.

Why: R002 requires `anim_graph.ron` to load as a typed Bevy asset through the animation module, not only parse as a standalone serde struct. This task closes the S01 demo with real Agumon data while preserving the generic module boundary required by R001.

Do:
1. Add `src/animation/plugin.rs` and export it from `src/animation/mod.rs`.
2. Define a small `AnimationAssetPlugin` that registers `RonAssetPlugin::<AnimGraph>::new(&["ron"])`, loads animation graph paths through `AssetServer`, and tracks handles/load state using Bevy 0.18 APIs (`MessageReader<AssetEvent<AnimGraph>>`) patterned after `src/data/mod.rs`.
3. Add a constant or configurable resource for graph asset paths. For S01 it can include only `digimon/agumon/anim_graph.ron`, but the API should be data-driven and ready for S04 non-Agumon paths without adding new Rust enum variants or branches.
4. Add `assets/digimon/agumon/anim_graph.ron` using the T01 schema. Model a minimal real `baby_flame` flow using identifiers from `assets/data/digimon/agumon/skills.ron` and animation names/frame intent from `assets/digimon/agumon_atlas.json`; keep semantic cross-asset validation for S03.
5. Add `tests/anim_graph_asset.rs` that builds a headless Bevy `App` with asset infrastructure plus `AnimationAssetPlugin`, advances update ticks until the Agumon graph asset is present or a bounded timeout is hit, and asserts the loaded asset's `clip`, `entry`, node, command, predicate, and target-shape data are typed as expected.
6. Avoid adding `windowed`, UI, or rendering dependencies to these tests.

Done when: The real Agumon `anim_graph.ron` loads through `AnimationAssetPlugin` as an `AnimGraph` asset in a headless test, and loader state cannot report ready before the graph has produced a load/modified event and can be read from `Assets<AnimGraph>`.

## Inputs

- `src/animation/mod.rs`
- `src/animation/anim_graph.rs`
- `tests/anim_graph_parse.rs`
- `src/data/mod.rs`
- `Cargo.toml`
- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon_atlas.json`

## Expected Output

- `src/animation/mod.rs`
- `src/animation/plugin.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_graph_asset.rs`

## Verification

cargo test --test anim_graph_asset

## Observability Impact

Runtime/load observability: introduce explicit handle/load-state resources and useful Bevy logs for loaded/modified animation graphs. Failure modes covered: missing asset, malformed RON, and silent readiness before an asset is actually available.
