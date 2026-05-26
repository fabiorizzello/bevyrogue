---
estimated_steps: 3
estimated_files: 1
skills_used: []
---

# T03: Add camera-shake cue writing Camera2d from a captured rest

Why: the milestone requires camera-shake to exist as just another registered cue (not a special case) that writes the Camera2d transform; it is the only genuinely new behaviour in S03 and carries the documented drift risk (must be absolute offset from rest, never accumulated — MEM094).

Do: In src/windowed/render.rs: (1) Add a CameraRest{ translation: Vec3 } component and attach it in setup_camera (~L431) capturing the Camera2d's spawn translation, so the shake restores to the exact rest without drift (same anti-drift pattern as SpriteRest). (2) Add a binary-local CameraShakeState resource (single remaining: u32, mirroring HitShakeState's shape) and init it in RenderPlugin::build. Arm it on the SAME CombatEvent::OnHitTaken signal that arms HitShakeState — add a small windowed observer/system reading CombatEvent and setting CameraShakeState.remaining to the camera_impact cue's ticks (look up "camera_impact" in CueRegistry). Emit a trace! on target "windowed.agumon_playback" when armed so a future agent can confirm it fired (observability — auto-mode can't run the binary). (3) Decay CameraShakeState in the SINGLE existing decay block in advance_agumon_presentation (the if pending_ticks.0 > 0 block at ~L807 — do NOT add a second decay site, MEM094). (4) Add an apply system (ordered after decay) that, when CameraShakeState.remaining > 0, looks up "camera_impact" in CueRegistry and sets camera_transform.translation = rest.translation + shake_offset_parametric(remaining, ticks, amp, freq_x, freq_y).extend(0.0)'s xy (absolute offset from CameraRest), and hard-sets translation back to rest.translation when remaining == 0. Use the CameraShake variant params from the registry.

Done when: cargo build --features windowed exits 0 zero warnings; cargo test --features windowed --test windowed_only green; grep confirms CameraRest, CameraShakeState, and a Camera2d-writing shake apply system are present in render.rs.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/ui/cues.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed
