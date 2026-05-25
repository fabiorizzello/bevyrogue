---
id: T01
parent: S05
milestone: M004
key_files:
  - src/windowed/render.rs
  - assets/digimon/agumon/vfx.ron
  - tests/windowed_only/vfx_rendering_acceptance.rs
  - tests/windowed_only.rs
  - tests/animation/vfx_asset_load.rs
key_decisions:
  - Used Bevy 0.18's `Hdr` component as the local equivalent of the plan's `hdr: true` requirement.
  - Switched windowed VFX color writes to `Color::linear_rgba` so overbright asset channels drive HDR bloom instead of being treated as UI-style sRGB values.
duration: 
verification_result: passed
completed_at: 2026-05-25T18:13:01.813Z
blocker_discovered: false
---

# T01: Enabled windowed HDR bloom camera policy and added contract coverage that fails on missing HDR/bloom camera setup or clamped Agumon VFX color data.

**Enabled windowed HDR bloom camera policy and added contract coverage that fails on missing HDR/bloom camera setup or clamped Agumon VFX color data.**

## What Happened

Updated `src/windowed/render.rs` so the windowed camera now spawns with explicit HDR/bloom post-processing policy (`Camera2d`, `Hdr`, `Bloom::NATURAL`, `Tonemapping::TonyMcMapface`, `DebandDither::Enabled`) and VFX particle colors are written with `Color::linear_rgba` so authored channels above 1.0 survive into HDR rendering. Re-authored `assets/digimon/agumon/vfx.ron` comments and Baby Flame/Baby Burner bloom-facing color curves to use intentional overbright linear intensity for charge, projectile, impact, and flash effects. Added `tests/windowed_only/vfx_rendering_acceptance.rs` and wired it into the aggregated `windowed_only` harness so future regressions fail fast if the camera loses HDR/bloom/tonemapping/dithering or if Agumon bloom-capable effects are clamped back to <= 1.0 RGB. Updated exact asset-load assertions in `tests/animation/vfx_asset_load.rs` to match the new HDR-authored curve values.

## Verification

Ran the required windowed compile and targeted acceptance test. `cargo check --features windowed` passed after repairing a transient duplicated file tail introduced during editing. `cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture` passed with both acceptance checks green: camera HDR/bloom policy and overbright authored VFX data.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features windowed` | 0 | ✅ pass | 411ms |
| 2 | `cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture` | 0 | ✅ pass | 1189ms |

## Deviations

Bevy 0.18 does not expose the older `Camera { hdr: true }` field expected by the task text, so the implementation used the equivalent local API contract: explicit `Hdr` component plus Bloom/Tonemapping/DebandDither.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/render.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/windowed_only/vfx_rendering_acceptance.rs`
- `tests/windowed_only.rs`
- `tests/animation/vfx_asset_load.rs`
