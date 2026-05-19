# S01 — Research

**Date:** 2026-05-18

## Summary

S01 should introduce a new cohesive animation module seam rather than adding another data-only type beside `src/data`. The existing project pattern for typed RON assets is in `src/data/mod.rs`: register `RonAssetPlugin::<T>::new(&["ron"])`, load asset paths with `AssetServer`, and observe `AssetEvent::LoadedWithDependencies` / `Modified` in Update systems. M001's architecture decisions require this pattern to be reused from behind an animation boundary, not by scattering `AnimGraph` schema ownership across combat/data modules.

The main technical risk is schema shape, not Bevy loading. The canonical design in `docs/future_design_draft/02-02b_animation_fsm.md` defines `AnimGraph` as a closed-vocabulary FSM: graph `clip`, `entry`, `nodes`, and `transitions`; nodes carry frame ranges, optional playback modifier/reverse, and `on_enter` commands; edges carry closed predicates and priority. M001 adapts M022, so use the M022 roadmap as schema guidance but rename/locate code around the current repo's generic animation module requirement.

## Recommendation

Create `src/animation/` as the cohesive module boundary, with internal submodules such as `anim_graph.rs`, `plugin.rs`, and later `clip.rs`/`validation.rs`. Export a small `AnimationAssetPlugin` that registers `RonAssetPlugin::<AnimGraph>` and loads configured animation graph paths. Keep `src/data` as the existing combat data loader; do not place the generic FSM schema under `src/data` or `src/combat/blueprints/agumon`, because R001 explicitly requires a generic animation module without hardcoded Digimon rules.

Represent schema vocabularies as Rust enums with serde derives and no `Custom(String)` escape hatch for commands, predicates, param refs, or target shapes. This intentionally makes out-of-vocabulary RON fail during parse/deserialization, satisfying S01's rejection criterion. For the first proof, parse fixtures with `ron::from_str::<AnimGraph>` in a headless test, then add the Bevy asset registration/load proof once the schema compiles.

## Implementation Landscape

### Key Files

- `src/lib.rs` — currently exports `combat`, `data`, `party_validation`, `ui`; add `pub mod animation;` so tests and downstream slices import the new boundary directly.
- `src/data/mod.rs` — pattern to follow for `RonAssetPlugin`, `AssetServer::load`, handle resources, and `AssetEvent::LoadedWithDependencies`; avoid moving animation logic here except possibly high-level app composition later.
- `src/data/error.rs` — typed error style: errors should include asset path/check context and be diagnosable from logs. S01 parse failures can start as RON/serde failures in tests; S03 should wrap semantic validation in animation-specific diagnostics.
- `docs/future_design_draft/02-02b_animation_fsm.md` — canonical command, predicate, target-shape, and validator vocabulary source. Important excerpts: Commands are closed; numbers should be referenced through params instead of literals; target shapes are closed enums; no hidden scripting via string commands.
- `docs/M022/M022-ROADMAP.md` and `docs/M022/slices/S01/S01-PLAN.md` — prior plan names `src/combat/blueprints/anim_graph/types.rs`, but M001 should adapt that to `src/animation/` to respect the new module decision.
- `assets/data/digimon/agumon/skills.ron` — real Agumon skill id `baby_flame` and targeting data useful for authoring the first graph without inventing Digimon-specific engine logic.
- `assets/digimon/agumon_atlas.json` — source animation names and frame ranges; S01 needs graph `clip` names to align with S02/validator, especially `skill` for baby_flame.
- `tests/` — many integration tests are top-level; add a focused `tests/anim_graph_parse.rs` or module-level tests under `src/animation/anim_graph.rs` for deserialization and invalid vocabulary.

### Build Order

1. Add `src/animation/mod.rs` and `src/animation/anim_graph.rs` with the minimal closed schema needed for Agumon `baby_flame`: `AnimGraph`, `Node`, `Edge`, `NodeId`/newtype aliases, `Command`, `Predicate`, `ParamRef`, `TargetShape`, and optional modifier types.
2. Write the first headless unit/contract test that parses an inline valid RON graph and asserts `clip`, `entry`, node frame ranges, command variants, predicates, and priorities. This proves the highest-risk schema shape before Bevy lifecycle details.
3. Add an invalid RON fixture/inline string containing an unknown command/predicate/target shape and assert `ron::from_str::<AnimGraph>` returns an error. This directly satisfies the out-of-vocabulary acceptance path.
4. Add `AnimationAssetPlugin` / `AnimGraphHandles` only after pure parsing is stable, following `DataPlugin`'s `RonAssetPlugin::<T>::new(&["ron"])` and handle tracker pattern.
5. Author `assets/digimon/agumon/anim_graph.ron` after the schema is stable. Keep content generic: it may refer to Agumon skill ids/params as data, but Rust schema/loader must not special-case Agumon.

### Verification Approach

- `cargo test --test anim_graph_parse` if adding a top-level integration test.
- `cargo test animation::anim_graph` if colocating schema tests in the module.
- `cargo test` after asset plugin registration to catch feature/default regressions.
- `cargo check` and later `cargo check --features windowed` remain milestone-level guardrails; S01 should not add winit/wgpu/egui dependencies.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Typed RON asset loading | `bevy_common_assets::ron::RonAssetPlugin` already used by `DataPlugin` | Keeps animation assets on the same Bevy asset lifecycle and hot-reload path as current RON data. |
| RON parse tests | `ron::from_str` and serde derives already in dependencies | Fast headless schema tests without starting a full Bevy app. |
| Error derivation | `thiserror` already used in `src/data/error.rs` | Use for future animation validation diagnostics instead of stringly errors. |

## Constraints

- R001/R002: schema and loading belong behind one generic animation module boundary; no Digimon-specific Rust logic in the core module.
- R008: all S01 verification must run headless. Do not introduce `windowed` feature dependencies to schema or loader tests.
- Closed vocabularies are intentional. Avoid `Command::Custom(String)`, free-form predicate kinds, or untyped target shapes; unknown values should fail deserialization.
- M022 paths under `src/combat/blueprints/anim_graph` are historical guidance, not binding. Current M001 decisions prefer `src/animation` as the owner seam.

## Common Pitfalls

- **Letting asset paths define architecture** — existing runtime data paths live under `assets/data/...`, while sprite atlas JSON lives under `assets/digimon/...`. Animation assets can live with sprite assets, but Rust module ownership should still stay generic.
- **Overfitting Agumon baby_flame** — the first asset should be real Agumon data, but all Rust types must work for non-Agumon S04 assets without adding variants or hardcoded IDs.
- **Semantic checks too early** — S01 should reject unknown enum values through typed schema; graph reachability, missing clip refs, param existence, and frame in-bounds belong to S03 validator unless needed to make loader tests meaningful.
- **Mixing presentation and gameplay blindly** — `02-02b` allows commands that can be gameplay or cosmetic, but the schema only declares intent. Later adapters/blueprints translate; S01 should not implement kernel effects.

## Open Risks

- Exact final command vocabulary may be large. Implement the closed enum broadly enough for M022/M001 expectations, but planner should decide whether S01 needs all §C/§C2 variants immediately or can include variants as inert schema-only enum cases.
- Bevy 0.18 uses `MessageReader<AssetEvent<T>>` in current code; executor should mirror that API exactly rather than older `EventReader` examples from Bevy docs.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy | `bevy` | installed in available skills; relevant for plugin/assets/ECS patterns |
| Rust | `rust-best-practices`, `rust-testing` | installed in available skills; relevant for idiomatic schema/errors/tests |
| API/interface design | `api-design`, `design-an-interface` | installed in available skills; principles support closed, evolvable schema surface |
| Observability | `observability` | installed in available skills; supports typed diagnostics/log-readable failure surfaces |

## Sources

- Existing Bevy RON asset lifecycle pattern: `src/data/mod.rs`.
- Existing typed data error style: `src/data/error.rs`.
- Canonical animation FSM schema/vocabulary: `docs/future_design_draft/02-02b_animation_fsm.md`.
- M022 historical slice plan and roadmap: `docs/M022/M022-ROADMAP.md`, `docs/M022/slices/S01/S01-PLAN.md`.
- Real Agumon skill and atlas inputs: `assets/data/digimon/agumon/skills.ron`, `assets/digimon/agumon_atlas.json`.
