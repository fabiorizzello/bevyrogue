# S01: Animation module and anim graph schema

**Goal:** Create the generic animation module seam, closed typed AnimGraph schema, real Agumon anim_graph.ron asset, and headless tests proving valid typed loading plus out-of-vocabulary rejection.
**Demo:** `cargo test` loads an Agumon `anim_graph.ron` as a typed asset through the new animation module and rejects out-of-vocabulary schema values with typed errors.

## Must-Haves

- Owned requirements: R001 and R002. Supporting requirement: R008.
- Done means all of the following are true:
- `src/lib.rs` exposes `pub mod animation;` and animation schema/loading lives under `src/animation/` rather than `src/data` or Digimon-specific modules.
- `AnimGraph` deserializes from RON using closed Rust enums for nodes, transitions, predicates, commands, parameter references, and target shapes; unknown enum variants fail at parse time.
- `assets/digimon/agumon/anim_graph.ron` exists and models Agumon baby_flame with generic data only, no Agumon-specific Rust branches.
- `AnimationAssetPlugin` registers `RonAssetPlugin::<AnimGraph>::new(&["ron"])`, loads configured graph asset paths relative to `assets/`, and exposes handle/load state resources following the current `DataPlugin` Bevy 0.18 `MessageReader<AssetEvent<T>>` pattern.
- Headless verification passes: `cargo test --test anim_graph_parse`, `cargo test --test anim_graph_asset`, and `cargo test`.

## Proof Level

- This slice proves: Contract plus headless integration. This slice proves typed schema contracts and Bevy asset loading registration without running the windowed runtime. Real runtime/UAT is not required until S04.

## Integration Closure

Upstream surfaces consumed: `src/data/mod.rs` for the existing typed RON asset lifecycle pattern, `docs/future_design_draft/02-02b_animation_fsm.md` for closed FSM vocabulary, `assets/data/digimon/agumon/skills.ron` and `assets/digimon/agumon_atlas.json` for real Agumon identifiers/frame names. New wiring introduced: `src/animation` module exported from `src/lib.rs` and an `AnimationAssetPlugin` that can later be composed into app startup. Remaining milestone work: S02 typed clip loading, S03 semantic/cross-asset validation, S04 non-Agumon coverage plus windowed hot-reload UAT.

## Verification

- Q3 Threat Surface: the only untrusted input is local RON asset content; exploit impact is boot/test denial via malformed assets, not secrets or external data exposure. Q4 Requirement Impact: touches R001, R002, R008 and preserves decisions D001-D004. Q5 Failure Modes: malformed RON or unknown schema values must produce deterministic parse/test failures; asset load events must not silently mark graphs ready. Q6 Load Profile: expected graph count is small and loaded at boot, per-operation cost is one RON asset parse per graph; 10x roster scale should stress asset count/log volume before CPU. Q7 Negative Tests: include unknown command/predicate/target-shape RON cases and an asset-loading test that asserts the real Agumon graph becomes available as an `AnimGraph` asset.

## Tasks

- [x] **T01: Define closed AnimGraph schema and parse contracts** `est:2h`
  Skills expected in executor frontmatter: bevy, rust-best-practices, rust-testing, api-design, design-an-interface, tdd, verify-before-complete.
  - Files: `src/lib.rs`, `src/animation/mod.rs`, `src/animation/anim_graph.rs`, `tests/anim_graph_parse.rs`
  - Verify: cargo test --test anim_graph_parse

- [x] **T02: Wire AnimGraph asset plugin and real Agumon graph** `est:2h`
  Skills expected in executor frontmatter: bevy, rust-best-practices, rust-testing, tdd, verify-before-complete.
  - Files: `src/animation/mod.rs`, `src/animation/plugin.rs`, `assets/digimon/agumon/anim_graph.ron`, `tests/anim_graph_asset.rs`
  - Verify: cargo test --test anim_graph_asset

- [x] **T03: Run full headless regression and tighten integration edges** `est:1h`
  Skills expected in executor frontmatter: bevy, rust-testing, verify-before-complete.
  - Files: `src/lib.rs`, `src/animation/mod.rs`, `src/animation/anim_graph.rs`, `src/animation/plugin.rs`, `assets/digimon/agumon/anim_graph.ron`, `tests/anim_graph_parse.rs`, `tests/anim_graph_asset.rs`
  - Verify: cargo test

## Files Likely Touched

- src/lib.rs
- src/animation/mod.rs
- src/animation/anim_graph.rs
- tests/anim_graph_parse.rs
- src/animation/plugin.rs
- assets/digimon/agumon/anim_graph.ron
- tests/anim_graph_asset.rs
