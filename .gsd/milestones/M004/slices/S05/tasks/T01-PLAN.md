---
estimated_steps: 5
estimated_files: 3
skills_used: []
---

# T01: Enable HDR bloom rendering policy

Expected executor skills: bevy, rust-development, rust-testing, tdd, verify-before-complete.

Why: S05 must close the automated HDR/bloom rendering acceptance gap before visual UAT. Current `setup_camera` spawns a bare `Camera2d`, and authored VFX colors are not bloom-capable. This task implements the low-risk D037 path: HDR camera + Bloom/Tonemapping/DebandDither plus overbright data-driven VFX colors, without claiming true custom additive material.

Do: First add or extend a windowed-only contract test that fails if `setup_camera` lacks `hdr: true`, `Bloom`, `Tonemapping`, or `DebandDither`, and if real Agumon VFX contains no color channel above `1.0` for bloom-capable effects. Then update `src/windowed/render.rs` with Bevy 0.18 camera imports/components following the local 2D bloom example. Update `assets/digimon/agumon/vfx.ron` comments and relevant Baby Flame/Baby Burner appearance keyframes so VFX color channels may intentionally exceed `1.0` as intensity, not clamped UI color. Keep all rendering imports windowed-gated and do not move Bevy render types into headless animation modules.

Done when: Windowed compile succeeds, the new rendering-acceptance test proves HDR/bloom configuration and overbright VFX data, and no strict additive material claim is introduced.

Quality gates: Q3 threat surface is low because this is local rendering only with no auth, network, secrets, or user input. Q4 requirement impact touches milestone-local R002/R005 rendering gating and supports R004 by keeping math headless. Q5 failure mode is Bevy API drift: compile/test failures must name missing camera components. Q6 load profile is per-camera/per-particle rendering; no shared backend resource. Q7 negative coverage should fail if HDR/bloom tokens are removed or if all VFX RGB channels are clamped to <=1.0.

## Inputs

- `src/windowed/render.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/windowed_only/vfx_asset_impact_render.rs`

## Expected Output

- `src/windowed/render.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/windowed_only/vfx_rendering_acceptance.rs`

## Verification

cargo check --features windowed
cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture

## Observability Impact

Adds executable/static diagnostics for camera bloom configuration and bloom-capable asset color policy, giving future agents a precise failure surface before any manual window launch.
