# S05: Sharp Claws and rendering acceptance remediation — UAT

**Milestone:** M004
**Written:** 2026-05-25T18:29:12.624Z

# S05 UAT: Sharp Claws VFX and Rendering Acceptance

**UAT Type:** Automated contract + windowed compile proof. Human visual signoff is NOT part of S05 — that belongs to S06.

## Preconditions

- `cargo` toolchain available with the `windowed` feature flag
- `assets/digimon/agumon/vfx.ron` and `assets/digimon/agumon/anim_graph.ron` present
- `assets/vfx/sharp_claws_slash.png` present (64×64 RGBA baked-orientation texture)
- `.gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md` present and non-empty

## Test Steps

1. **Verify acceptance artifact exists:**
   ```
   test -s .gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md
   ```
   Expected: exit code 0.

2. **Run headless asset-load contracts:**
   ```
   cargo test --test animation vfx_asset_load -- --nocapture
   ```
   Expected: 16 passed, 0 failed. Key passing tests: `agumon_vfx_contains_sharp_claws_slash` (asset has `sharp_claws.slash` using the registered `agumon/baby_flame/static` verb, single particle, ttl 6, TargetCenter, size 34.0).

3. **Run headless curve-eval contracts:**
   ```
   cargo test --test animation vfx_asset_eval -- --nocapture
   ```
   Expected: 12 passed, 0 failed. Key passing test: `sharp_claws_slash_curves_evaluate_deterministically_and_overbright` — all RGB channels >1.0 (HDR-bloom proxy), alpha fade 0.95→0.0, 1000 evals bit-identical.

4. **Run no-VFX-kind regression guard:**
   ```
   cargo test --test animation render_no_vfx_kind_guard -- --nocapture
   ```
   Expected: 2 passed. Guard asserts: no `VfxParticleKind` / `kind_from_name` identifier in `render.rs`; `on_enter_effect_ids` boundary still present and routes `sharp_claws_slash` through the owned effect id.

5. **Windowed compile check:**
   ```
   cargo check --features windowed
   ```
   Expected: exit 0, `Finished dev profile`.

6. **Windowed impact render contracts:**
   ```
   cargo test --features windowed --test windowed_only vfx_asset_impact_render -- --nocapture
   ```
   Expected: 7 passed.

7. **Windowed HDR/bloom acceptance:**
   ```
   cargo test --features windowed --test windowed_only vfx_rendering_acceptance -- --nocapture
   ```
   Expected: 2 passed. Tests: camera has HDR+Bloom+Tonemapping+DebandDither; Agumon VFX colors are overbright (all RGB >1.0).

## Expected Outcomes

- All 7 commands exit 0 with the test counts above.
- `M004-RENDERING-ACCEPTANCE.md` explicitly states the D037 deferral (no strict additive material) and the K001 boundary (no `cargo winx` human signoff).
- `render.rs` contains no `VfxParticleKind` enum or `kind_from_name` string-match dispatch.
- `sharp_claws.slash` effect in `vfx.ron` reuses `agumon/baby_flame/static` placement verb with no new verb registration.

## Edge Cases

- **Near-miss name routing:** `on_enter_effect_ids("sharp_claws")`, `on_enter_effect_ids("slash")`, `on_enter_effect_ids("")` must NOT resolve to `AGUMON_SHARP_CLAWS_EFFECT_ID`. Covered by the unit test.
- **Stale verb removal:** If `agumon/baby_flame/static` is ever deregistered, `vfx_asset_load::agumon_vfx_contains_sharp_claws_slash` will fail immediately, localizing the regression.
- **Overbright color clamp:** If vfx.ron colors are changed to <1.0, `vfx_rendering_acceptance::agumon_vfx_keeps_bloom_capable_overbright_color_channels` fails, localizing the bloom regression.

## Not Proven By This UAT

- Human visual quality of the Sharp Claws slash effect (claw streak shape, bloom glow intensity, frame timing) — S06 owns `cargo winx` human signoff or waiver.
- Strict custom additive particle blending — D037 explicitly defers this; Bevy 0.18 2D ColorMaterial has no true additive mode.
