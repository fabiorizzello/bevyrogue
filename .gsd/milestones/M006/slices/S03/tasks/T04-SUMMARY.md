---
id: T04
parent: S03
milestone: M006
key_files:
  - tests/windowed_only/digimon_sprite_cue_dispatch.rs
  - tests/windowed_only.rs
key_decisions:
  - Used code-shaped tokens so surviving comments referencing old names cannot trip presence/absence checks (MEM101)
  - Asserted only structural token presence/absence, never formulas, so the contract survives parametric-math refactors
  - include_str! both render.rs and mod.rs from one test module, mirroring enoki_impact_render.rs
duration: 
verification_result: passed
completed_at: 2026-05-26T11:33:51.080Z
blocker_discovered: false
---

# T04: Added windowed source-contract test pinning the generalized DigimonSprite + CueRegistry dispatch seams

**Added windowed source-contract test pinning the generalized DigimonSprite + CueRegistry dispatch seams**

## What Happened

Created tests/windowed_only/digimon_sprite_cue_dispatch.rs as an include_str! source-contract test (the only automated guard for binary-crate windowed wiring per MEM030/MEM101/K001) and registered it in tests/windowed_only.rs with a #[path] mod line in the existing aggregator style. Five tests pin the S03 seams that must outlive the S04 agumon extraction: (1) render.rs defines struct DigimonSprite and enum DigimonPlaybackMode with no struct AgumonSprite/enum AgumonPlaybackMode tokens; (2) DigimonSprite carries stance_graph_id/skill_graph_id as data fields; (3) the feedback path references CueRegistry and calls flash_tint_parametric/shake_offset_parametric while the legacy flash_tint and shake_offset lib calls are absent; (4) CameraRest, CameraShakeState, Camera2d, and a mut Transform camera write are present; (5) mod.rs references CueRegistry and the three cue ids hit_flash/hit_shake/camera_impact. All assertions are code-shaped presence/absence tokens, no formulas, so comments mentioning old names don't trip checks.

## Verification

Ran cargo test --features windowed --test windowed_only — exit 0, 59 passed / 0 failed. Confirmed the 5 new digimon_sprite_cue_dispatch tests are present and pass via a grep-filtered rerun (exit 0).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only` | 0 | pass | 1072ms |
| 2 | `cargo test --features windowed --test windowed_only | grep -E digimon_sprite_cue_dispatch` | 0 | pass | 327ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/windowed_only/digimon_sprite_cue_dispatch.rs`
- `tests/windowed_only.rs`
