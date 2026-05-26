---
id: T02
parent: S03
milestone: M005
key_files:
  - src/windowed/render.rs
key_decisions:
  - Detect freshly-armed units for the trace by checking remaining == FLASH_TICKS before the once-per-frame decay (gated on pending_ticks.0 > 0), so the 'flash+shake armed' trace fires exactly once per arming rather than every 0-tick render frame.
  - Hard-set transform.translation absolutely from SpriteRest.xy every tick (offset when shaking, rest when not) instead of add/subtract deltas — eliminates drift and is idempotent.
  - Guard the flash color write with death_exiting.is_none() && fade_out.is_none() so the flash (sole AgumonSprite color writer) never competes with advance_death_fade's alpha lerp.
duration: 
verification_result: passed
completed_at: 2026-05-26T09:02:28.539Z
blocker_discovered: false
---

# T02: Wired the windowed flash tint + positional shake onto the struck AgumonSprite, driven from the OnHitTaken-armed HitFlashState/HitShakeState lib projection.

**Wired the windowed flash tint + positional shake onto the struck AgumonSprite, driven from the OnHitTaken-armed HitFlashState/HitShakeState lib projection.**

## What Happened

Applied T01's pure hit-feedback projection to the binary so a struck sprite visibly flashes and shakes (the K001 half), following S01's drive_hurt_reactions registration pattern.

In RenderPlugin::build: init_resource::<HitFlashState>()/HitShakeState>() and registered bevyrogue::ui::hit_feedback::observe_hit_feedback ordered .after(spawn_unit_sprites).after(resolve_action_system).before(advance_agumon_presentation).before(continue_suspended_timeline_system), mirroring drive_hurt_reactions so the windows are armed before they are decayed/applied.

Added a binary-local SpriteRest { xy: Vec2 } component and insert it in spawn_unit_sprites capturing the spawn (x, 0.0), so the shake restores to the exact rest position without drift (research warns the hardcoded ±200 layout value goes stale; capture at spawn instead).

In advance_agumon_presentation: added ResMut<HitFlashState>/ResMut<HitShakeState> params; changed the p0 query's &Transform to &mut Transform and added &SpriteRest. Once per frame (gated on pending_ticks.0 > 0) the system traces freshly-armed units (those still at FLASH_TICKS before decay) with trace!(target: "windowed.agumon_playback", unit_id, "flash+shake armed") then decays both windows by pending_ticks.0 — a single decay source of truth fed from the same PendingAnimationTicks the presentation runs on. Per sprite each tick it writes render_sprite.color = flash_tint(remaining, FLASH_TICKS) (flash is the sole color writer; WHITE at remaining 0 keeps steady state white) GUARDED to skip when death_exiting.is_some() || fade_out.is_some() so it never fights advance_death_fade's alpha lerp; and sets transform.translation to (rest.xy + shake_offset(remaining, SHAKE_TICKS)).extend(z) as an absolute offset, hard-setting back to rest.xy.extend(z) when shake remaining is 0 (never accumulated).

These stay pure overlays: sprite.mode, sprite.player, and the kernel barrier are untouched (D031/D032). A dropped/duplicated OnHitTaken is idempotent (re-arms to full); a hit on a UnitId with no live sprite just decays unused. The brief sub-second shake perturbs VFX anchors while active (sprite_positions reads the shaken Transform) — documented and accepted per S03-RESEARCH.

## Verification

cargo build --features windowed compiles clean (7.26s). cargo test --features windowed is green: 0 failures across all binaries (the windowed_only binary's 42 tests pass including the T01 windowed_hit_feedback cases; the only non-zero non-pass count is a single ignored test, not a failure). cargo clippy --features windowed introduces no new warnings — render.rs:633 too-many-arguments (15/7) and the complex-ParamSet-type warning are pre-existing on advance_agumon_presentation (already 13 args + ParamSet before this edit; clippy emits one warning per function regardless of arg count, so the warning count is unchanged). Visible flash + shake appearance is K001 (manual cargo winx) and was not run from auto mode.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 7260ms |
| 2 | `cargo test --features windowed` | 0 | pass | 0ms |
| 3 | `cargo clippy --features windowed` | 0 | pass (no new warnings; pre-existing workspace lints only) | 5450ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
