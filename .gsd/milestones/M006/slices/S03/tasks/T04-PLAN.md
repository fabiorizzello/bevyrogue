---
estimated_steps: 3
estimated_files: 2
skills_used: []
---

# T04: Add windowed source-contract tests pinning the generalized seams

Why: src/windowed/ is binary-crate code unreachable from tests/ (MEM030); the only automated guards for this wiring are windowed source-contract tests (include_str! token assertions on render.rs/mod.rs, per MEM101) plus build green. These tests are the slice's objective stopping condition and must outlive the S04 extraction.

Do: Create tests/windowed_only/digimon_sprite_cue_dispatch.rs (mirroring the include_str! pattern in enoki_impact_render.rs / vfx_windowed_contracts.rs) with assertions that: (a) render.rs defines struct DigimonSprite and enum DigimonPlaybackMode and contains NO AgumonSprite/AgumonPlaybackMode tokens (use code-shaped tokens like 'struct DigimonSprite' and absence of 'struct AgumonSprite' / 'enum AgumonPlaybackMode' to avoid false matches in comments — MEM101/S01 pattern); (b) DigimonSprite carries the graph ids as data — assert 'stance_graph_id' and 'skill_graph_id' fields are present; (c) the flash/shake path reads the registry — assert render.rs references CueRegistry and calls flash_tint_parametric and shake_offset_parametric, and that the legacy 'flash_tint(' / 'shake_offset(' lib calls are absent (code-shaped); (d) camera-shake exists and writes the camera — assert CameraRest, CameraShakeState, and Camera2d transform write tokens are present; (e) mod.rs registers the three cue ids — assert 'hit_flash', 'hit_shake', 'camera_impact' string literals and 'CueRegistry' appear in mod.rs (include_str! of mod.rs). Register the new module in tests/windowed_only.rs with a #[path] mod line matching the existing aggregator style. Do not assert exact formulas — assert presence/absence of structural tokens only.

Done when: cargo test --features windowed --test windowed_only is green with the new test module included and all new assertions pass.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `tests/windowed_only.rs`
- `tests/windowed_only/enoki_impact_render.rs`

## Expected Output

- `tests/windowed_only/digimon_sprite_cue_dispatch.rs`
- `tests/windowed_only.rs`

## Verification

cargo test --features windowed --test windowed_only
