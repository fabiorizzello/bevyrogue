# S01: S01 — UAT

**Milestone:** M001
**Written:** 2026-05-18T20:51:37.747Z

# UAT Type
Headless contract and asset-loading verification.

# Preconditions
- Repository is checked out at the S01 closeout state.
- Rust toolchain and project dependencies are installed.
- The real graph fixture exists at `assets/digimon/agumon/anim_graph.ron`.

# Steps
1. Run `cargo test --test anim_graph_parse`.
2. Observe the parse-contract suite covering a valid graph plus unknown command, predicate, and target-shape variants.
3. Run `cargo test --test anim_graph_asset`.
4. Observe the asset-loading test bootstrap a headless Bevy app with `AnimationAssetPlugin` and load the Agumon graph.
5. Run `cargo test`.

# Expected Outcomes
1. The parse-contract suite passes and shows that valid `AnimGraph` RON deserializes into closed typed variants.
2. Unknown schema vocabulary fails during parsing rather than being accepted as free-form strings.
3. The asset-loading suite passes and shows the real Agumon graph becomes readable as an `AnimGraph` asset.
4. Loader readiness remains false until both a qualifying asset event occurs and the typed asset can be fetched from `Assets<AnimGraph>`.
5. Full headless regression passes with no failures.

# Edge Cases
- Replace a command, predicate, or target-shape enum value in a test fixture with an unknown variant: the parse suite should fail deterministically.
- Break the Agumon graph path or file contents: the asset-loading test should fail rather than report ready.

# Not Proven By This UAT
- Typed `clip.ron` loading or geometry parity.
- Cross-asset semantic validation beyond typed graph loading.
- Windowed runtime behavior or hot reload proof; those remain for later slices.
