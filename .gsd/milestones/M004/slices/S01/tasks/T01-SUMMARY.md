---
id: T01
parent: S01
milestone: M004
key_files:
  - src/animation/vfx_asset.rs
  - src/animation/mod.rs
  - tests/animation/vfx_asset_schema.rs
  - tests/animation.rs
key_decisions:
  - VfxAsset omits the TypePath derive because Reflect's derive supplies it (E0119 otherwise); Asset only needs TypePath to exist
  - No Cargo.toml change needed — bevy_reflect is a hard dep of bevy_internal so reflection resolves under default-features=false; the bevy facade exposes no bevy_reflect feature
  - EffectId is a transparent String newtype (Eq+Ord+Hash) serving as the effect map key and on_expire chain reference
  - Floats (f32) in the schema mean Eq/Hash are intentionally dropped from the appearance/curve types (unlike anim_graph which has no floats)
duration: 
verification_result: passed
completed_at: 2026-05-25T10:01:47.311Z
blocker_discovered: false
---

# T01: Added editor-ready typed VfxAsset RON schema (Serialize+Deserialize+Reflect, deny_unknown_fields) with 3 passing schema tests

**Added editor-ready typed VfxAsset RON schema (Serialize+Deserialize+Reflect, deny_unknown_fields) with 3 passing schema tests**

## What Happened

Created the new headless module `src/animation/vfx_asset.rs` defining the owned per-Digimon VFX schema per D033/D034. Top-level `VfxAsset { effects: BTreeMap<EffectId, EffectDef> }` derives `Asset, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect` with `#[serde(deny_unknown_fields)]`. `EffectDef` carries a typed `Placement { verb: String }` (namespaced Registry id by string; resolution deferred to S02), an `Appearance`, and an optional `on_expire: Option<EffectId>` chain reference expressed in data. `Appearance` holds the typed, introspectable fields `count: u32`, `spread_px: f32`, `ttl_ticks: u32`, `scale: ScaleCurve`, `color: ColorCurve`. `ScaleCurve(Vec<ScaleKeyframe>)` and `ColorCurve(Vec<ColorKeyframe>)` are transparent newtypes; `ScaleKeyframe { t, value }` and `ColorKeyframe { t, rgba: [f32;4] }` each derive the full set. `EffectId` is a transparent String newtype with Eq+Ord+Hash so it serves as the map key. Every type derives Reflect for D034 editor-readiness. Exported via `pub mod vfx_asset; pub use vfx_asset::*;` in `src/animation/mod.rs`.

Two plan deviations, both forced by the bevy 0.18 API and documented below:
1. The plan's contingency said "if Reflect does not resolve, add the bevy_reflect feature to Cargo.toml". I tried that first; cargo rejected it — the bevy facade has no `bevy_reflect` feature (it is a hard, non-optional dep of bevy_internal). `bevy::prelude::Reflect` and `bevy::reflect::Struct` already resolve under default-features=false with no feature change, so Cargo.toml was left unchanged.
2. Deriving both `TypePath` and `Reflect` on VfxAsset caused E0119 (Reflect's derive already emits a TypePath impl). Dropped the explicit `TypePath` from the derive list; `Asset` only requires that TypePath exist, which Reflect supplies. Captured as MEM070.

No windowed dependency introduced (R016): the module is pure data + serde + reflect, lives in the headless lib, and compiles under the default headless feature set. R004 is respected — `ttl_ticks` is tick-based, no wall-clock.

## Verification

Ran `cargo test --test animation vfx_asset_schema` (default headless features). The lib compiled clean and all 3 schema tests passed: (a) `vfx_asset_round_trips_through_ron` — inline RON deserialize -> serialize -> deserialize equality; (b) `unknown_field_is_rejected` — RON with an extra `bogus_field` fails to deserialize, proving deny_unknown_fields; (c) `appearance_is_reflectable_with_expected_fields` — `<Appearance as bevy::reflect::Struct>` field-name introspection via name_at/field_len returns ["count","spread_px","ttl_ticks","scale","color"], proving D034 editor-readiness. The unknown-field negative test is the Q3 malformed-RON threat-surface check.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation vfx_asset_schema` | 0 | pass — 3 passed; 0 failed (65 filtered out) | 5638ms |

## Deviations

Two forced deviations, both documented: (1) did NOT add a bevy_reflect feature to Cargo.toml (the plan's contingency) because no such facade feature exists and reflection already resolves; (2) dropped the explicit TypePath derive from VfxAsset to avoid an E0119 conflict with Reflect's generated TypePath impl.

## Known Issues

none

## Files Created/Modified

- `src/animation/vfx_asset.rs`
- `src/animation/mod.rs`
- `tests/animation/vfx_asset_schema.rs`
- `tests/animation.rs`
