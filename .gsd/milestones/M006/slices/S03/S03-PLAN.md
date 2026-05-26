# S03: Generalize windowed sprite + wire cue dispatch

**Goal:** Generalize the Agumon-named windowed presentation component into a data-carrying DigimonSprite (stance/skill graph ids become fields, not const lookups) and route transient flash/sprite-shake/camera-shake feedback through the S02 CueRegistry + parametric math instead of the hit_feedback consts — adding camera-shake as just another registered cue that writes the Camera2d transform. AGUMON_* consts, AgumonAtlas, and Agumon-specific cue registration stay in the engine (their extraction is S04).
**Demo:** In cargo winx, hits still flash/shake the struck sprite, camera-shake fires on impact, and stance/skill/hurt/death playback still works — now driven by the generic DigimonSprite + cue dispatch instead of Agumon-named components and hit_feedback consts. Windowed regression suite green.

## Must-Haves

- cargo build --features windowed exits 0 with zero warnings; cargo test --features windowed --test windowed_only is green (count rises from 54 with new contract tests); cargo test --test dependency_gating stays 2/2 (no enoki/bevy_color leak headless); render.rs contains DigimonSprite/DigimonPlaybackMode and zero AgumonSprite/AgumonPlaybackMode identifiers; the legacy lib flash_tint(/shake_offset( calls are gone from the binary (replaced by CueRegistry-sourced *_parametric); a camera-shake cue writes Camera2d translation as an absolute offset from a captured CameraRest. K001 manual: cargo winx shows hits still flash/shake the struck sprite, camera-shake fires on impact, and stance/skill/hurt/death playback is unchanged.

## Proof Level

- This slice proves: contract + integration (windowed source-contract tests + build green); runtime visual is K001 manual — auto-mode cannot run the windowed binary (MEM030).

## Integration Closure

Upstream consumed: CueRegistry/CueDef + flash_tint_parametric/shake_offset_parametric/SrgbTriple from src/ui/cues.rs (S02); single enoki path + AgumonEnokiVfx map (S01); HitFlashState/HitShakeState/observe_hit_feedback from src/ui/hit_feedback.rs. New wiring: .init_resource::<CueRegistry>() + registration of hit_flash/hit_shake/camera_impact defs in src/windowed/mod.rs; CameraRest capture in setup_camera; CameraShakeState arm/decay + Camera2d-writing apply system in render.rs. Remaining before milestone usable: S04 extracts AGUMON_* consts + cue registration into src/windowed/digimon/agumon/; S05 adds Renamon.

## Verification

- Existing trace! on target windowed.agumon_playback "flash+shake armed" stays (now reports registry-sourced ticks); camera-shake arm should emit an analogous trace on the same target so a future agent can confirm it fired without running the binary. No new failure states — all paths are pure presentation overlays that never touch CombatState (R010).

## Tasks

- [x] **T01: Generalize AgumonSprite to data-carrying DigimonSprite** `est:2h`
  Why: extension-first presentation requires the windowed component to carry stance/skill graph ids as DATA so S04/S05 add Digimon without editing it, and the milestone requires zero AgumonSprite/AgumonPlaybackMode identifiers to remain. This is the highest-blast-radius seam (S03-RESEARCH seam 1) — land it first and confirm green before wiring cues.
  - Files: `src/windowed/render.rs`
  - Verify: cargo build --features windowed

- [x] **T02: Register CueRegistry and re-point flash/shake to parametric math** `est:1h30m`
  Why: the slice's 'After this' requires flash/shake to be driven by cue dispatch reading CueRegistry instead of hit_feedback consts (D048 model a — param sourcing, behaviour-preserving). The S02 parametric fns are already proven bit-for-bit identical to the legacy fns at the legacy params, so this is a behaviour-preserving swap.
  - Files: `src/windowed/mod.rs`, `src/windowed/render.rs`
  - Verify: cargo build --features windowed

- [x] **T03: Add camera-shake cue writing Camera2d from a captured rest** `est:1h30m`
  Why: the milestone requires camera-shake to exist as just another registered cue (not a special case) that writes the Camera2d transform; it is the only genuinely new behaviour in S03 and carries the documented drift risk (must be absolute offset from rest, never accumulated — MEM094).
  - Files: `src/windowed/render.rs`
  - Verify: cargo build --features windowed

- [x] **T04: Add windowed source-contract tests pinning the generalized seams** `est:1h`
  Why: src/windowed/ is binary-crate code unreachable from tests/ (MEM030); the only automated guards for this wiring are windowed source-contract tests (include_str! token assertions on render.rs/mod.rs, per MEM101) plus build green. These tests are the slice's objective stopping condition and must outlive the S04 extraction.
  - Files: `tests/windowed_only/digimon_sprite_cue_dispatch.rs`, `tests/windowed_only.rs`
  - Verify: cargo test --features windowed --test windowed_only

## Files Likely Touched

- src/windowed/render.rs
- src/windowed/mod.rs
- tests/windowed_only/digimon_sprite_cue_dispatch.rs
- tests/windowed_only.rs
