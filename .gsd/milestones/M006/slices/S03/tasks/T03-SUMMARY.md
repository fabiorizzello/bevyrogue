---
id: T03
parent: S03
milestone: M006
key_files:
  - src/windowed/render.rs
key_decisions:
  - CameraShakeState is a single global u32 remaining (mirroring HitShakeState's shape) since there is exactly one camera, not a per-unit map
  - Camera-shake decay lives in the existing single PendingAnimationTicks decay block in advance_agumon_presentation; apply_camera_shake is a separate system ordered after it to read the drained remaining — no second decay site (MEM094)
  - Camera transform is written as an absolute offset from a captured CameraRest (rest.translation + offset, snapping to rest at remaining 0), never additive, preventing drift
  - Arm reuses the OnHitTaken hit_damage_amount signal identical to observe_hit_feedback with its own MessageReader cursor; cue params are sourced from the camera_impact CueRegistry entry registered in T02
duration: 
verification_result: passed
completed_at: 2026-05-26T11:31:27.761Z
blocker_discovered: false
---

# T03: Added camera-shake as a registered CueRegistry cue that writes the Camera2d transform as an absolute offset from a captured CameraRest (no drift)

**Added camera-shake as a registered CueRegistry cue that writes the Camera2d transform as an absolute offset from a captured CameraRest (no drift)**

## What Happened

Implemented camera-shake in src/windowed/render.rs as just another registered cue — the only genuinely new behaviour in S03 — reusing the S02 CueRegistry parametric math and the existing single-decay discipline.

1. CameraRest component — added `CameraRest { translation: Vec3 }` and attached it in `setup_camera`, capturing the camera's spawn translation (made explicit via `let transform = Transform::default()`). Mirrors `SpriteRest`: the apply system restores to the exact rest rather than hardcoding a layout value, so the camera never accumulates drift (MEM094).

2. CameraShakeState resource — binary-local `CameraShakeState { remaining: u32 }` (single global counter; mirrors HitShakeState's shape but there is one camera). init_resource in RenderPlugin::build. arm(ticks) (idempotent reset, collapses same-window multi-hit) and decay_by(n) (saturating).

3. Arming — observe_camera_shake reads CombatEvent (own cursor, MEM065), arms on the SAME OnHitTaken signal as HitShakeState (hit_damage_amount(..).is_some()), looks up camera_impact in CueRegistry and sizes the window to that cue's ticks. Emits trace! on target windowed.agumon_playback ("camera-shake armed", target_unit + camera_shake_ticks) so a future agent can confirm it fired without running the binary (K001). Ordered like observe_hit_feedback.

4. Decay — added camera_shake.decay_by(pending_ticks.0) to the SINGLE existing `if pending_ticks.0 > 0` decay block in advance_agumon_presentation (next to flash/shake decays); no second decay site (MEM094). Added mut camera_shake: ResMut<CameraShakeState> param.

5. Apply — apply_camera_shake, ordered .after(advance_agumon_presentation) to read the drained remaining. remaining>0 → transform.translation = rest.translation + shake_offset_parametric(remaining, ticks, amp, freq_x, freq_y).extend(0.0) (absolute offset, z from rest); remaining==0 → hard-set translation = rest.translation. Falls through to no offset on a missing/wrong def. Queries (&mut Transform, &CameraRest), carried only by the Camera2d entity.

The camera_impact cue was already registered in T02 (CameraShake amp 4.0, freq_x 1.7, freq_y 2.3, ticks 8). All paths are pure presentation overlays, never touching CombatState (R010). CombatEvent was qualified as bevyrogue::combat::events::CombatEvent to match the file's convention.

## Verification

cargo build --features windowed exits 0 with zero warnings; cargo test --features windowed --test windowed_only is green (54 passed, 0 failed); grep confirms struct CameraRest, struct CameraShakeState, the observe_camera_shake arm system, and the apply_camera_shake Camera2d-writing system (Query<(&mut Transform, &CameraRest)>) are present in render.rs. Per K001 the windowed binary was NOT run — the "camera-shake armed" trace on target windowed.agumon_playback is the manual-confirmation seam.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 2640ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass (54 passed; 0 failed) | 20000ms |
| 3 | `rg -n 'struct CameraRest|struct CameraShakeState|fn apply_camera_shake|fn observe_camera_shake' src/windowed/render.rs` | 0 | pass (all symbols present) | 50ms |

## Deviations

Added an explicit Transform::default() binding in setup_camera (Camera2d already requires Transform; explicit binding lets CameraRest capture the exact spawn translation). Qualified CombatEvent rather than adding a top-level import, matching the file's convention.

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
