---
sliceId: S03
uatType: artifact-driven
verdict: PASS
date: 2026-05-26T11:50:00.000Z
---

# UAT Result — S03

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `cargo build --features windowed` exit 0, zero warnings | artifact | PASS | Incremental build in 0.20s, exit 0. No warnings emitted. |
| windowed_only test suite: 59 passed / 0 failed | runtime | PASS | `cargo test --features windowed --test windowed_only` → 59 passed, 0 failed, 0.03s. |
| dependency_gating: 2 passed / 0 failed | runtime | PASS | `bevy_enoki_absent_from_headless_graph` + `bevy_enoki_present_in_windowed_graph` both ok. |
| DigimonSprite/DigimonPlaybackMode present in render.rs (33 hits) | artifact | PASS | `grep -c "DigimonSprite\|DigimonPlaybackMode"` → 33 in src/windowed/render.rs. |
| AgumonSprite/AgumonPlaybackMode absent in render.rs (0 hits) | artifact | PASS | `grep -c "AgumonSprite\|AgumonPlaybackMode"` → 0. Rename is complete. |
| stance_graph_id / skill_graph_id data fields present | artifact | PASS | Lines 54 + 57 define fields; lines 200–212 and 1282/1347/1424 read them at call sites. |
| flash_tint_parametric / shake_offset_parametric calls present (4 total) | artifact | PASS | 4 call sites in render.rs (lines 567, 1037, 1052 + import at line 27). |
| Legacy flash_tint( / shake_offset( calls absent (0 hits) | artifact | PASS | `grep "flash_tint(\|shake_offset("` → 0 matches in render.rs. |
| CameraRest struct present | artifact | PASS | Defined at line 135. |
| CameraShakeState struct present | artifact | PASS | Defined at line 144. |
| observe_camera_shake fn present | artifact | PASS | Defined at line 528. |
| apply_camera_shake fn present | artifact | PASS | Defined at line 554. |
| CueRegistry + hit_flash / hit_shake / camera_impact in mod.rs | artifact | PASS | 5 matches: init_resource at line 90, register_agumon_cues fn at 136, cue id strings at 139/146/155. |
| Source-contract test file exists with 5 tests | artifact | PASS | `tests/windowed_only/digimon_sprite_cue_dispatch.rs` (5228 bytes, 5 #[test] fns). |
| Observability seam: trace!("camera-shake armed") on windowed.agumon_playback | artifact | PASS | Line 541 in render.rs; target confirmed at line 538. |
| Observability seam: trace!("flash+shake armed") on windowed.agumon_playback | artifact | PASS | Line 957 in render.rs; target confirmed at line 953. |
| Step 1 — Stance/skill playback unchanged (K001 visual) | human-follow-up | NEEDS-HUMAN | Run `cargo run --features windowed`; observe idle cycle, Baby Flame windup→strike→recovery, hurt reaction. All should work identically to pre-S03. |
| Step 2 — Hit flash fires on impact (K001 visual) | human-follow-up | NEEDS-HUMAN | Land a hit; struck sprite should flash reddish tint ~8 frames, peak ≈ (1.0, 0.45, 0.45), then return to normal. |
| Step 3 — Sprite shake fires on impact (K001 visual) | human-follow-up | NEEDS-HUMAN | Land a hit; struck sprite should oscillate laterally ~8 frames, then snap to rest with no residual offset. |
| Step 4 — Camera shake fires on impact (K001 visual) | human-follow-up | NEEDS-HUMAN | Land a hit; entire camera view should jolt ~8 frames then snap cleanly back. No drift or progressive offset accumulation. |
| Step 5 — No drift after multiple rapid hits (K001 visual) | human-follow-up | NEEDS-HUMAN | Land 3–4 hits rapidly; camera and sprites should each return cleanly to rest position after each burst. |
| Step 6 — RUST_LOG trace confirmation (K001 optional) | human-follow-up | NEEDS-HUMAN | Run `RUST_LOG=windowed.agumon_playback=trace cargo run --features windowed`, land a hit; check terminal for "flash+shake armed" and "camera-shake armed". (Trace seam confirmed present in source at lines 541 and 957.) |

## Overall Verdict

PASS — all 16 automatable checks passed (build clean, 59/59 windowed_only, 2/2 dep-gating, all structural seams confirmed). 6 K001 visual/experiential checks remain NEEDS-HUMAN and cannot be automated per MEM030 (auto-mode cannot launch the windowed binary).

## Notes

- The UAT file explicitly declares this as `K001 Manual` — visual verification of flash/shake/camera-shake behavior requires a human to run `cargo run --features windowed` and observe the battle scene.
- Automated evidence covers: build cleanliness, full test suite pass, complete rename of AgumonSprite→DigimonSprite (33 hits / 0 legacy), data fields present at definition and all call sites, parametric math wired (4 call sites, 0 legacy calls), CameraRest/CameraShakeState/observers all present at expected lines, CueRegistry registration with all 3 cue ids, 5 source-contract tests registered and passing, and both trace observability seams confirmed at source level.
- The trace seam at render.rs:538–541 (`target: "windowed.agumon_playback"`, message "camera-shake armed") provides a machine-readable signal that a human can verify with `RUST_LOG=windowed.agumon_playback=trace` without re-reading the binary source.
