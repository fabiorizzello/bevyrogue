---
id: S03
parent: M006
milestone: M006
provides:
  - Generic DigimonSprite component (graph ids as data fields) replacing AgumonSprite
  - CueRegistry init + Agumon cue registration (hit_flash/hit_shake/camera_impact) in UiPlugin
  - flash_tint_parametric/shake_offset_parametric replacing legacy lib calls in render.rs
  - Camera-shake cue: CameraRest + CameraShakeState + observe_camera_shake + apply_camera_shake writing Camera2d
  - 5 windowed source-contract tests pinning S03 seams (digimon_sprite_cue_dispatch.rs)
requires:
  - slice: S02
    provides: CueRegistry/CueDef + flash_tint_parametric/shake_offset_parametric/SrgbTriple
  - slice: S01
    provides: single enoki spawn path + AgumonEnokiVfx map
affects:
  []
key_files: []
key_decisions:
  - DigimonSprite carries stance_graph_id/skill_graph_id as String data fields; spawn site passes AGUMON_*_GRAPH_ID consts as data source (extraction deferred to S04)
  - CueRegistry init + register_agumon_cues Startup system live in UiPlugin (mod.rs); cue ids hit_flash/hit_shake/camera_impact registered with legacy const values for behaviour-preserving swap
  - Camera-shake decay runs in the single existing pending_ticks decay block in advance_agumon_presentation — no second decay site (MEM094)
  - CameraRest captures spawn translation at setup_camera; apply_camera_shake writes absolute offset (rest + parametric), never additive — prevents drift
  - Source-contract tests use code-shaped include_str! token assertions; no formulas so tests survive parametric-math refactors (MEM101)
patterns_established:
  - DigimonSprite data-carrying pattern: add Digimon by spawning with different stance_graph_id/skill_graph_id — zero render.rs edits required
  - CueRegistry parametric dispatch: flash/shake/camera cues are pure data lookups; adding a new cue type requires only a CueDef variant + registry entry
  - Single decay site discipline: all per-frame decay (flash, sprite-shake, camera-shake) runs in one `if pending_ticks.0 > 0` block — never split across systems
  - Windowed source-contract test pattern: include_str! token assertions in tests/windowed_only/ guard binary-crate seams that are invisible to normal tests (MEM030/MEM101)
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-26T11:35:57.620Z
blocker_discovered: false
---

# S03: Generalize windowed sprite + wire cue dispatch

**Generalized AgumonSprite to data-carrying DigimonSprite, routed flash/shake/camera-shake through CueRegistry parametric math, and pinned all seams with 5 new source-contract tests (59 windowed_only, 2/2 dep-gating, zero warnings).**

## What Happened

Four tasks delivered the slice goal end-to-end.

**T01 — Generalize AgumonSprite to data-carrying DigimonSprite.**
Renamed `AgumonSprite → DigimonSprite` and `AgumonPlaybackMode → DigimonPlaybackMode` throughout src/windowed/render.rs. Added `stance_graph_id: String` and `skill_graph_id: String` data fields so S04/S05 can spawn new Digimon without editing render.rs. All resolve_snapshot call sites were switched to read ids off the queried sprite instead of module consts; the spawn site still passes the AGUMON_*_GRAPH_ID consts as the data source (extraction deferred to S04 per plan). Verification: 0 AgumonSprite/AgumonPlaybackMode hits in render.rs; 33 DigimonSprite/DigimonPlaybackMode hits; cargo build --features windowed + windowed_only 54/54 green.

**T02 — Register CueRegistry and re-point flash/shake to parametric math.**
Added `.init_resource::<CueRegistry>()` and a `register_agumon_cues` Startup system to UiPlugin in src/windowed/mod.rs, registering `CueDef::Flash { peak:(1.0,0.45,0.45), ticks:8 }` under "hit_flash", `CueDef::SpriteShake { amp:4.0, freq_x:1.7, freq_y:2.3, ticks:8 }` under "hit_shake", and `CueDef::CameraShake` under "camera_impact" (consumed by T03). In render.rs, replaced the two legacy lib calls in advance_agumon_presentation with CueRegistry lookups + `flash_tint_parametric`/`shake_offset_parametric` calls — a behaviour-preserving swap (D048 model a) proven bit-for-bit identical at these params by S02 unit tests. Legacy `flash_tint(`/`shake_offset(` calls: 0; parametric calls: 5. cargo build + windowed_only 54/54 + dependency_gating 2/2 green.

**T03 — Add camera-shake cue writing Camera2d from a captured CameraRest.**
Added `CameraRest { translation: Vec3 }` component captured at camera spawn in `setup_camera`. Added `CameraShakeState { remaining: u32 }` resource init in RenderPlugin::build. `observe_camera_shake` arms on the same OnHitTaken signal as HitShakeState (own MessageReader cursor; MEM065), looks up "camera_impact" in CueRegistry for the tick window, and emits `trace!(target: "windowed.agumon_playback", "camera-shake armed")` for K001 manual confirmation. Decay runs in the single existing `if pending_ticks.0 > 0` block (MEM094 — no second decay site). `apply_camera_shake`, ordered after advance_agumon_presentation, writes `transform.translation = rest.translation + offset` (absolute, no drift) or snaps to rest at remaining==0. All paths are pure presentation overlays that never touch CombatState (R010). Build + 54/54 windowed_only green.

**T04 — Source-contract tests pinning the generalized seams.**
Created tests/windowed_only/digimon_sprite_cue_dispatch.rs (include_str! source-contract pattern per MEM030/MEM101) with 5 tests: (1) DigimonSprite/DigimonPlaybackMode present, AgumonSprite/AgumonPlaybackMode absent in render.rs; (2) stance_graph_id/skill_graph_id data fields present; (3) CueRegistry + parametric calls present, legacy lib calls absent; (4) CameraRest/CameraShakeState/Camera2d/mut Transform camera write present; (5) CueRegistry + hit_flash/hit_shake/camera_impact in mod.rs. Registered in tests/windowed_only.rs. Final count: 59 passed / 0 failed (up from 54).

## Verification

1. `cargo build --features windowed` — exit 0, zero warnings (artifact cached; confirmed via zero-duration incremental rebuild after all tasks).
2. `cargo test --features windowed --test windowed_only` — exit 0, **59 passed / 0 failed** (up from 54; 5 new digimon_sprite_cue_dispatch contract tests added).
3. `cargo test --test dependency_gating` — exit 0, **2 passed / 0 failed** (bevy_enoki_absent_from_headless + bevy_enoki_present_in_windowed both green; no enoki leak into headless).
4. Structural grep: `DigimonSprite/DigimonPlaybackMode` 33 hits in render.rs; `AgumonSprite/AgumonPlaybackMode` 0 hits.
5. Legacy `flash_tint(`/`shake_offset(` calls in render.rs: 0; `flash_tint_parametric`/`shake_offset_parametric` calls: 5.
6. `struct CameraRest`, `struct CameraShakeState`, `fn observe_camera_shake`, `fn apply_camera_shake` all present in render.rs at lines 135/144/528/554.
7. CueRegistry + hit_flash/hit_shake/camera_impact references in mod.rs: 5 hits.
8. K001 (windowed binary run): deferred to manual sign-off per MEM030 (auto-mode cannot launch the windowed binary). Observability seam: `trace!(target: "windowed.agumon_playback", "camera-shake armed")` emitted on each arm so a human can confirm the signal without re-reading the binary.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

AGUMON_* consts and cue registration remain in the windowed engine (src/windowed/mod.rs + render.rs spawn site) — extraction to src/windowed/digimon/agumon/ is S04. K001 manual sign-off (visual confirmation of flash/shake/camera-shake in the running binary) is not automated.

## Follow-ups

S04: extract AGUMON_* consts + cue registration into src/windowed/digimon/agumon/register(app) so grep of engine files shows zero Agumon-specific identifiers.

## Files Created/Modified

- `src/windowed/render.rs` — 
- `src/windowed/mod.rs` — 
- `tests/windowed_only/digimon_sprite_cue_dispatch.rs` — 
- `tests/windowed_only.rs` — 
