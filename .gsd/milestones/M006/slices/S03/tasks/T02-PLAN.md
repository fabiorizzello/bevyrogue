---
estimated_steps: 3
estimated_files: 2
skills_used: []
---

# T02: Register CueRegistry and re-point flash/shake to parametric math

Why: the slice's 'After this' requires flash/shake to be driven by cue dispatch reading CueRegistry instead of hit_feedback consts (D048 model a — param sourcing, behaviour-preserving). The S02 parametric fns are already proven bit-for-bit identical to the legacy fns at the legacy params, so this is a behaviour-preserving swap.

Do: (1) In src/windowed/mod.rs add .init_resource::<bevyrogue::ui::cues::CueRegistry>() and a startup system (or build-time population) that registers exactly three Agumon cue defs with the legacy const values: CueDef::Flash{ peak:(1.0,0.45,0.45), ticks:8 } under id "hit_flash"; CueDef::SpriteShake{ amp:4.0, freq_x:1.7, freq_y:2.3, ticks:8 } under id "hit_shake"; CueDef::CameraShake{ amp:4.0, freq_x:1.7, freq_y:2.3, ticks:8 } under id "camera_impact" (T03 consumes camera_impact). Registration must be collision-free (D047 panics on conflicting def). In S03 this registration is still Agumon-specific and lives in the engine; S04 moves it into the agumon module. (2) In src/windowed/render.rs advance_agumon_presentation, replace the legacy flash_tint(remaining, FLASH_TICKS) call (~L888) and shake_offset(shake_remaining, SHAKE_TICKS) call (~L895) with CueRegistry lookups: get("hit_flash")/get("hit_shake"), destructure the CueDef params, call flash_tint_parametric(remaining, ticks, peak) and shake_offset_parametric(remaining, ticks, amp, freq_x, freq_y), and map the returned SrgbTriple verbatim to Color::srgb(r,g,b) (MEM113/MEM114). Preserve MEM094 discipline exactly: flash is the sole colour writer and is skipped under DeathExiting/FadeOut; shake is an absolute offset from rest.xy, hard-set back to rest at remaining 0; both windows still decay once per frame on PendingAnimationTicks (single decay source of truth). Add the CueRegistry as a Res param to advance_agumon_presentation. The arming/decay (observe_hit_feedback, HitFlashState/HitShakeState) is unchanged. Drop the now-unused FLASH_TICKS/SHAKE_TICKS/flash_tint/shake_offset imports only if no longer referenced (the 'flash+shake armed' trace at ~L809 may still use the tick consts — keep them if so).

Done when: cargo build --features windowed exits 0 zero warnings; cargo test --features windowed --test windowed_only green; cargo test --test dependency_gating stays 2/2; grep confirms the legacy flash_tint(/shake_offset( lib calls are gone from render.rs (replaced by *_parametric) and CueRegistry is referenced in render.rs and mod.rs.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/ui/cues.rs`
- `src/ui/hit_feedback.rs`

## Expected Output

- `src/windowed/mod.rs`
- `src/windowed/render.rs`

## Verification

cargo build --features windowed
