---
id: T02
parent: S05
milestone: M004
key_files:
  - assets/digimon/agumon/vfx.ron
  - assets/digimon/agumon/anim_graph.ron
  - src/windowed/render.rs
  - assets/vfx/sharp_claws_slash.png
  - tests/animation/vfx_asset_load.rs
  - tests/windowed_only/vfx_asset_impact_render.rs
  - tests/animation/vfx_asset_schema.rs
key_decisions:
  - Reused the registered agumon/baby_flame/static placement verb for sharp_claws.slash rather than adding a new verb — keeps it RON-only with no register_agumon_ext/core change (respects D037 and the S05 acceptance bar).
  - Used TargetCenter anchor + a single particle whose claw orientation is baked into the texture, because windowed particle rendering has no per-particle rotation.
  - Brought four stale T01-era color/count test assertions in line with the committed overbright HDR RON values instead of reverting the RON, since the overbright colors are the intended S05 bloom policy.
duration: 
verification_result: passed
completed_at: 2026-05-25T18:21:20.566Z
blocker_discovered: false
---

# T02: Authored data-driven Sharp Claws slash VFX (RON effect + AnimGraph trigger + render bridge + texture), reusing the static placement verb with no new kind branching

**Authored data-driven Sharp Claws slash VFX (RON effect + AnimGraph trigger + render bridge + texture), reusing the static placement verb with no new kind branching**

## What Happened

Added the `sharp_claws.slash` effect to `assets/digimon/agumon/vfx.ron`: a single target-anchored particle on the existing registered `agumon/baby_flame/static` verb, ttl 6 ticks, size 34px, a quick scale pop (0.6->1.0 by 0.3 life) then hold, and an overbright pale yellow-white color (3.0,3.0,2.2) that alpha-fades 0.95->0.0 so the windowed HDR+bloom camera blooms it. No on_expire. This respects D037 (no strict custom additive material) and reuses an already-registered placement verb (no register_agumon_ext/core change).

Wired the AnimGraph trigger by adding an `on_enter` `SpawnParticle(name: "sharp_claws_slash", origin: TargetCenter, motion: Static)` to the `sharp_claws_strike` node in `assets/digimon/agumon/anim_graph.ron`, using the existing node-entry bridge (no new local-frame cue plumbing).

Extended `src/windowed/render.rs` with the `AGUMON_SHARP_CLAWS_EFFECT_ID = "sharp_claws.slash"` constant, a `"sharp_claws_slash" => &[AGUMON_SHARP_CLAWS_EFFECT_ID]` arm in `on_enter_effect_ids`, a `"sharp_claws_slash" => visuals.sharp_claws_slash.clone()` arm in `vfx_texture_handle`, and a `sharp_claws_slash` field on `VfxVisuals` loaded from `vfx/sharp_claws_slash.png` — all through the existing string->effect/texture maps, with no new VfxParticleKind-style branching.

Generated `assets/vfx/sharp_claws_slash.png` (64x64 RGBA) containing three oriented diagonal claw streaks; the orientation is baked into the texture because windowed particle rendering has no per-particle rotation.

Deviation: discovered four pre-existing stale color assertions left behind by S05/T01's overbright HDR color change to vfx.ron — assets were committed overbright but expectations still held old clamped values, so the suite was red before my work. Updated them to match the committed RON.

## Verification

Ran cargo check --features windowed (exit 0). Ran cargo test --test animation vfx_asset_load (15 passed) and the full cargo test --test animation harness (110 passed) confirming the new effect parses, round-trips, and validates against the registered verb set. Ran cargo test --features windowed --lib (20 passed) and cargo test --features windowed --test windowed_only (34 passed) confirming the render bridge compiles and the data-driven impact/effect contracts hold. No window was launched (K001).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features windowed` | 0 | pass | 700ms |
| 2 | `cargo test --test animation vfx_asset_load` | 0 | pass (15 passed) | 1500ms |
| 3 | `cargo test --test animation` | 0 | pass (110 passed) | 2000ms |
| 4 | `cargo test --features windowed --test windowed_only` | 0 | pass (34 passed) | 3000ms |

## Deviations

Fixed pre-existing stale test assertions that S05/T01's overbright HDR color change to vfx.ron had left red (it committed overbright assets without updating expectations): charge/impact_flash/baby_burner.flash colors in tests/animation/vfx_asset_load.rs, impact shard color in tests/windowed_only/vfx_asset_impact_render.rs, and the effect-count (7->8) in tests/animation/vfx_asset_schema.rs.

## Known Issues

none

## Files Created/Modified

- `assets/digimon/agumon/vfx.ron`
- `assets/digimon/agumon/anim_graph.ron`
- `src/windowed/render.rs`
- `assets/vfx/sharp_claws_slash.png`
- `tests/animation/vfx_asset_load.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `tests/animation/vfx_asset_schema.rs`
