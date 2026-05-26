---
id: T02
parent: S03
milestone: M006
key_files:
  - src/windowed/mod.rs
  - src/windowed/render.rs
key_decisions:
  - CueRegistry init_resource + register_agumon_cues Startup system live in UiPlugin (mod.rs) per plan; registration stays Agumon-specific in the engine for S03 (S04 extracts it)
  - Flash lookup uses `if let Some(CueDef::Flash{..})` and shake uses a match guarded by shake_remaining>0; both fall through to no-op/ZERO on a missing/wrong cue def — harmless since the registry is always populated at Startup before Update runs
  - Kept FLASH_TICKS/SHAKE_TICKS imports: the 'flash+shake armed' trace both detects freshly-armed units (== FLASH_TICKS, the value HitFlashState::arm uses) and reports the tick window, so the consts are still referenced
duration: 
verification_result: passed
completed_at: 2026-05-26T11:27:24.836Z
blocker_discovered: false
---

# T02: Routed windowed flash/sprite-shake through CueRegistry parametric math and registered the three Agumon cues (hit_flash, hit_shake, camera_impact) with legacy const values

**Routed windowed flash/sprite-shake through CueRegistry parametric math and registered the three Agumon cues (hit_flash, hit_shake, camera_impact) with legacy const values**

## What Happened

Added `.init_resource::<bevyrogue::ui::cues::CueRegistry>()` plus a `register_agumon_cues` Startup system to UiPlugin in src/windowed/mod.rs. That system registers exactly three cue defs with the legacy hit_feedback const values: `CueDef::Flash { peak:(1.0,0.45,0.45), ticks:8 }` under "hit_flash", `CueDef::SpriteShake { amp:4.0, freq_x:1.7, freq_y:2.3, ticks:8 }` under "hit_shake", and `CueDef::CameraShake { amp:4.0, freq_x:1.7, freq_y:2.3, ticks:8 }` under "camera_impact" (T03 consumes the latter). Registration is collision-free per D047.

In src/windowed/render.rs, added `cue_registry: Res<CueRegistry>` to `advance_agumon_presentation` and replaced the two legacy lib calls inside the per-tick loop: the flash now looks up CueDef::Flash from the registry, calls `flash_tint_parametric(remaining, ticks, peak)`, and maps the returned SrgbTriple verbatim to `Color::srgb(r,g,b)` (MEM113/MEM114); the shake looks up CueDef::SpriteShake and calls `shake_offset_parametric(remaining, ticks, amp, freq_x, freq_y)`. MEM094 discipline preserved exactly — flash stays the sole colour writer and is still skipped under DeathExiting/FadeOut; shake remains an absolute offset from rest.xy, hard-set back to rest at remaining 0; both windows still decay once per frame on PendingAnimationTicks (single decay source of truth). The flash+shake-armed trace and arm/decay path (observe_hit_feedback, HitFlashState/HitShakeState) are unchanged. Dropped the now-unused flash_tint/shake_offset imports; FLASH_TICKS/SHAKE_TICKS imports stay because the "flash+shake armed" trace still reports and detects on those tick consts (per plan).

This is a behaviour-preserving param-sourcing swap (D048 model a): the S02 parametric fns are already unit-proven bit-for-bit identical to the legacy fns at these exact params.

## Verification

cargo build --features windowed: exit 0, zero warnings (verified via touch+rebuild grep for warning|error = empty). cargo test --features windowed --test windowed_only: 54 passed / 0 failed. cargo test --test dependency_gating: 2 passed / 0 failed (R005 headless graph stays enoki/render-free). grep confirms no legacy flash_tint(/shake_offset( calls remain in render.rs (only *_parametric), and CueRegistry is referenced in both render.rs (3x) and mod.rs (2x).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass — zero warnings | 2270ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass — 54/54 | 7496ms |
| 3 | `cargo test --test dependency_gating` | 0 | pass — 2/2 | 290ms |
| 4 | `grep -nE 'flash_tint\(|shake_offset\(' src/windowed/render.rs` | 1 | pass — no legacy lib calls remain | 6ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/windowed/mod.rs`
- `src/windowed/render.rs`
