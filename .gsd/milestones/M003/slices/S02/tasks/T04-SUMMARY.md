---
id: T04
parent: S02
milestone: M003
key_files:
  - src/windowed/render.rs
  - src/combat/runtime/cue_barrier.rs
key_decisions:
  - Animation playback runs on a fixed-rate clock (default 12 fps, BEVYROGUE_ANIM_FPS override) decoupled from render framerate; per-animation speed differences remain expressible per-node via PlaybackModifier::SpeedMul, and a per-Digimon base fps can later live in ClipMeta.
  - CueBarrierStatus carries source: UnitId so presentation bridges can gate on the caster; the barrier stays globally scoped in the kernel.
  - SPRITE_DISPLAY_SCALE = 0.4 is provisional; a multi-slot layout for 4-per-team is still owed (deferred).
duration: 
verification_result: passed
completed_at: 2026-05-22T11:36:33.601Z
blocker_discovered: false
---

# T04: Added a fixed-rate animation clock, caster-gated the cue barrier, and scaled sprites to 0.4 — visual-readiness fixes surfaced by manual K001

**Added a fixed-rate animation clock, caster-gated the cue barrier, and scaled sprites to 0.4 — visual-readiness fixes surfaced by manual K001**

## What Happened

Three windowed-only gaps surfaced during the manual K001 loop on the bridge and were fixed across separate sessions, then folded into the slice contract via this task. (1) Animation clock: advance_agumon_presentation previously stepped the AnimGraphPlayer once per render frame (~60fps), so the 6-frame idle cycled in ~0.1s and attack clips flashed sub-0.15s. Added an AnimationClock resource that accumulates Time::delta and advances the player once per 1/fps; default 12 fps, override via BEVYROGUE_ANIM_FPS; catch-up capped at 4 ticks; non-positive fps never ticks. Barrier release continues to sample the impact frame only on an animation tick. (2) Caster gating: the kernel cue barrier is global and CueBarrierStatus did not carry the caster, so both the ally and the dummy sprite entered Skill mode and animated an attack only one launched. Added source: UnitId to CueBarrierStatus (populated from inflight.action.source) and a pure barrier_targets_sprite(status, unit_id) predicate; sync_agumon_mode and annotate_active_animation now early-out unless the barrier belongs to that sprite. (3) Sprite scale: native 512px frames filled the viewport; added SPRITE_DISPLAY_SCALE applied via Transform::with_scale at spawn, tuned to 0.4 (~205px) on user feedback after an initial 0.2, provisional pending a multi-slot 4-per-team layout. Note: the lib build absorbed a parallel-session fix bundling combat_panel's MessageWriters into a tuple (the panel had hit Bevy's 16 system-param limit); that fix is not part of this task.

## Verification

cargo build --features windowed exits 0 clean. cargo test --features windowed --bin bevyrogue is 12/12 green, including the unit tests added by this work: anim_clock_accumulates_render_frames_into_anim_ticks, anim_clock_caps_catchup_after_a_hitch, anim_clock_with_nonpositive_fps_never_ticks, parse_anim_fps_defaults_and_validates, and barrier_targets_only_the_casting_sprite. Visual smoothness and on-screen scale confirmed by the user at runtime (idle no longer frantic; only the casting sprite animates; 0.4 scale legible). The windowed binary was not launched by auto-mode per K001.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed 2>&1 | tail -10` | 0 | pass | 0ms |
| 2 | `cargo test --features windowed --bin bevyrogue 2>&1 | tail -20` | 0 | pass (12/12) | 800ms |

## Deviations

This task did not exist in the original S02 plan; it was added via gsd_replan_slice to capture work done during the manual K001 loop. The sprite scale was tuned twice (0.2 then 0.4) on user feedback.

## Known Issues

No hurt/flinch animation on the damage target: clip.ron defines a hurt range (46-52) but anim_graph.ron has no hurt node and no reaction bridge drives the target's player to it on incoming damage — unbuilt work, not a bug. Sprite layout is single-slot per side at scale 0.4; a 4-per-team multi-slot layout is not yet implemented. Baby Flame and Baby Burner still do not animate (their release bridge is T03, still pending).

## Files Created/Modified

- `src/windowed/render.rs`
- `src/combat/runtime/cue_barrier.rs`
