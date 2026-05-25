---
estimated_steps: 5
estimated_files: 4
skills_used: []
---

# T03: Harden Sharp Claws and no-hardcoding contracts

Expected executor skills: rust-development, rust-testing, tdd, verify-before-complete.

Why: S05 cannot rely on visual/manual inspection; it needs automated proof that Sharp Claws is truly data-driven, validates through the real asset schema, maps through the render bridge, and does not regress the no-hardcoded VFX-kind acceptance guard.

Do: Extend headless animation tests so the real Agumon VFX asset must include `sharp_claws.slash`, the effect uses a known placement verb, has bounded/short TTL and spawn count, contains the expected texture key, and preserves deterministic scale/color evaluation including overbright bloom intensity. Update naming in `tests/animation/vfx_asset_load.rs` if existing names imply only Baby Flame/Baby Burner counts. Extend `src/windowed/render.rs` unit tests for `on_enter_effect_ids("sharp_claws_slash")` and for unrelated names returning no Sharp Claws effect. Keep `tests/animation/render_no_vfx_kind_guard.rs` green and add a token assertion if needed so neither `VfxParticleKind` nor `kind_from_name` returns.

Done when: Headless tests prove the Sharp Claws asset contract and render mapping without reading `.gsd/`, and the static guard still prevents old hardcoded VFX-kind paths.

Quality gates: Q3 no external attack surface. Q4 retests M004 no-hardcoded-VFX and local R004 deterministic math. Q5 malformed asset behavior should be covered through existing `validate_effects` rejection paths. Q6 tests should remain small file reads only. Q7 negative tests include at minimum unknown particle name mapping and preservation of the no-kind/no-string-match guard.

## Inputs

- `assets/digimon/agumon/vfx.ron`
- `src/animation/vfx_asset.rs`
- `src/animation/placement.rs`
- `src/windowed/render.rs`
- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `assets/vfx/sharp_claws_slash.png`

## Expected Output

- `tests/animation/vfx_asset_load.rs`
- `tests/animation/vfx_asset_eval.rs`
- `tests/animation/render_no_vfx_kind_guard.rs`
- `src/windowed/render.rs`

## Verification

cargo test --test animation vfx_asset_load -- --nocapture
cargo test --test animation vfx_asset_eval -- --nocapture
cargo test --test animation render_no_vfx_kind_guard -- --nocapture

## Observability Impact

Improves failure localization through targeted asset/schema/render-bridge assertions; future agents will see whether failure is missing RON, unknown verb, missing effect bridge, or hardcoded-path regression.
