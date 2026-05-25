---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T01: Define editor-ready typed VfxAsset schema (Serialize+Deserialize+Reflect, deny_unknown_fields)

Why: D033/D034 require an owned, editor-ready per-Digimon VFX schema whose verb parameters are typed and introspectable (NOT a stringly-typed map), so the future anim_graph+vfx GUI editor can generate forms by reflection without a schema refactor. This task lays the schema foundation and de-risks the Reflect derive early (no Reflect is currently used anywhere in src/). Do: create new headless module src/animation/vfx_asset.rs defining the typed schema: a top-level `VfxAsset` (a map of effect-id -> EffectDef) deriving #[derive(Asset, TypePath, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)] with #[serde(deny_unknown_fields)], mirroring the closed-vocabulary discipline in anim_graph.rs. Define `EffectDef` carrying: a placement reference (a typed `Placement { verb: String, ... }` newtype referencing a namespaced Registry id by string — Registry resolution itself is deferred to S02; here it is data only), an `Appearance` struct, and an optional `on_expire` chain reference (effect-id String) so chaining is expressed in data. Define `Appearance` with typed, introspectable fields: `count: u32`, `spread_px: f32`, `ttl_ticks: u32`, `scale: ScaleCurve`, `color: ColorCurve`. Define `ScaleCurve(Vec<ScaleKeyframe>)` and `ColorCurve(Vec<ColorKeyframe>)` where `ScaleKeyframe { t: f32, value: f32 }` and `ColorKeyframe { t: f32, rgba: [f32; 4] }` — every struct derives Serialize+Deserialize+Reflect. Export the new types from src/animation/mod.rs (add `pub mod vfx_asset;` and `pub use vfx_asset::*;`). If `bevy::prelude::Reflect` does not resolve in the headless lib, add the `bevy_reflect` feature to the bevy features list in Cargo.toml (default-features=false). Write tests/animation/vfx_asset_schema.rs and register it in tests/animation.rs: assert (a) an inline RON VfxAsset round-trips (deserialize -> serialize -> deserialize equality), (b) RON with an unknown field fails to deserialize (deny_unknown_fields), (c) the schema is reflectable (e.g. `<Appearance as bevy::reflect::Struct>` field-name introspection returns the expected field names, proving D034 editor-readiness). Done when: vfx_asset.rs compiles in the headless lib, the three schema tests pass, and no windowed dependency is introduced (R016). Threat surface (Q3): RON asset files are local/trusted content, not network input; the only untrusted-shaped surface is malformed RON, contained by deny_unknown_fields and the closed typed schema (negative test (b)). Requirement impact (Q4): governed by validated R004 (pure/headless) and R016 (no windowed leak); no requirement contract is broken; D033/D034 are applied, not revisited.

## Inputs

- `src/animation/anim_graph.rs`
- `src/animation/vfx.rs`
- `src/animation/mod.rs`
- `Cargo.toml`
- `tests/animation.rs`

## Expected Output

- `src/animation/vfx_asset.rs`
- `src/animation/mod.rs`
- `Cargo.toml`
- `tests/animation/vfx_asset_schema.rs`
- `tests/animation.rs`

## Verification

cargo test --test animation vfx_asset_schema
