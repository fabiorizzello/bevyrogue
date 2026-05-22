---
id: S02
parent: M003
milestone: M003
provides:
  - (none)
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - Bounce is VFX/gameplay-only — baby_flame_impact has no KernelCue self-loop; cast→impact→recover is linear by construction (MEM061, D031-adjacent).
  - should_auto_release_unbridged() = skill_start_node().is_none() — only skills not in {sharp_claws, baby_flame, agumon_ult} take the fallback; bridged skills release on their authored ReleaseKernel frame.
  - AnimationClock (12 fps default, BEVYROGUE_ANIM_FPS override) decouples sprite advancement from Bevy render rate; catch-up capped at 4 ticks; barrier release samples only on animation ticks (MEM062).
  - CueBarrierStatus.source: UnitId gates the presentation bridge — only the caster's sprite enters Skill mode; dummy stays idle during ally attacks.
  - Baby Flame parity tests assert against authored node union [60,77] not the coarse skill label [59,75] because baby_flame_recover overshoots to frames 76/77 (research finding #5, MEM057).
patterns_established:
  - skill_start_node() / classify_same_skill_sync() seams enable per-skill FSM entry and same-skill cue-hop without player reset — reusable for any future bridged skill.
  - ReleaseFrameKey (cue_id, node, local_frame) provides per-barrier dedup without hop_index since each Baby Burner barrier carries a distinct cue_id.
  - Pure fn barriers (barrier_targets_sprite, should_auto_release_unbridged, classify_same_skill_sync) keep windowed presentation logic unit-testable without constructing full ECS worlds.
observability_surfaces:
  - trace!(target: windowed.agumon_playback) logs skill_id + awaiting_cue_id on every presentation tick.
  - trace! on multi-barrier FSM advance (which barrier cue released, which node the player advanced into).
  - debug! on auto-release fallback narrowed to genuinely unbridged skills only.
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-22T12:12:44.138Z
blocker_discovered: false
---

# S02: S02

**Extended the windowed cue bridge to Baby Flame and Baby Burner — both skills now release on their authored ReleaseKernel frame with a fixed-rate animation clock, caster-gated barrier, and correctly scaled sprites.**

## What Happened

S02 extended the windowed Agumon presentation bridge from Sharp-Claws-only to cover Baby Flame (skill, range 59-75) and Baby Burner (ultimate, range 23-45), proving the impact-frame-on-rendered-frame invariant headless for both skills and delivering the visual-readiness fixes that K001 surfaced.

**T01 — Data authoring + headless invariant tests.** Three authoring gaps left Baby Flame and Baby Burner without rendered-frame release gating. `assets/data/digimon/agumon/skills.ron`: the Baby Flame impact_damage beat was changed from `presentation:None` to an explicit `Some((cue_id: "agumon/baby_flame/impact", anim: Some("baby_flame_impact"), …))` so the bridge could find its suspension point. `assets/digimon/agumon/anim_graph.ron`: added `cues: [(at:1, command: ReleaseKernel(()))]` to `baby_flame_impact` (clip frame 70, inside skill [59,75], mirrors sharp_claws_strike); added end-of-node ReleaseKernel cues to `baby_burner_charge` (at:7 → frame 30) and `baby_burner_recovery` (at:7 → frame 45) per D031. Four new atlas-parity tests in `tests/animation/atlas_binding.rs` — impact tile in-range + identity for Baby Flame and Baby Burner, plus full parity drives through each skill's node union — were green under `cargo test --test animation` (61/61) and full `cargo test` (zero headless regression).

**T02 — Pure seam generalization (Sharp Claws behavior preserved).** `AgumonPlaybackMode::SharpClaws{cue_id, presentation_node}` was replaced by `Skill{skill_id, awaiting_cue_id, start_node}` in `src/windowed/render.rs`. Two new pure seams were extracted: `skill_start_node(skill_id)` maps bridged skills to their FSM entry nodes (sharp_claws→sharp_claws_windup, baby_flame→baby_flame_cast, agumon_ult→baby_burner_charge; None for unbridged); `classify_same_skill_sync()` returns DifferentSkill|Unchanged|CueChanged. On a same-skill cue hop the FSM player advances without resetting — the key enabler for Baby Burner's three-barrier walk and Baby Flame's node progression. The lines-264-277 auto-release fallback was left unchanged (still gates only non-SharpClaws); Sharp Claws remained the only live bridge. `cargo build --features windowed` and `cargo test --features windowed` (7 render.rs unit tests, 23 harnesses total) were all green.

**T03 — Real bridges activated; bounce-to-FSM coupling removed.** The blanket non-SharpClaws auto-release was replaced by `should_auto_release_unbridged()` (= `skill_start_node().is_none()`), so only genuinely unbridged skills take the fallback. The pending-release path now drives the multi-barrier walk via `fire_kernel_cue()` — Baby Burner: charge→launch→recovery; Baby Flame: impact once then into recover. A post-completion manual K001 iteration revealed that the authored `baby_flame_impact` KernelCue self-loop (priority 10, intended for bounce re-entry) caused a visible extra windup on linear casts. Per explicit user direction — bounce is VFX/gameplay-only and must not touch the animation FSM — the self-loop was removed from `anim_graph.ron`; `baby_flame_impact` now has a single `TimeInNode→baby_flame_recover` edge and the path cast→impact→recover is linear by construction. Bounce scaffolding in render.rs (hop_index in ReleaseFrameKey, `kernel_cue_advances_node()` workaround) and mod.rs (`agumon::bouncing_fire` talent grant) was removed. All verification gates (animation tests 61/61, full suite, windowed build and tests) passed after the iteration.

**T04 — Visual-readiness fixes (three gaps surfaced by K001).** (1) AnimationClock resource: accumulates `Time::delta` and advances the AnimGraphPlayer once per `1/fps`; default 12 fps, `BEVYROGUE_ANIM_FPS` env override; catch-up capped at 4 ticks; non-positive fps never ticks. Barrier release samples only on animation ticks. (2) Caster gating: `source: UnitId` added to `CueBarrierStatus` (populated from `inflight.action.source`); `barrier_targets_sprite(status, unit_id)` predicate gates `sync_agumon_mode` and `annotate_active_animation` so only the casting sprite enters Skill mode. (3) Sprite scale: `SPRITE_DISPLAY_SCALE = 0.4` applied via `Transform::with_scale` at spawn (~205 px, provisional). 12 unit tests green (`cargo test --features windowed --bin bevyrogue`), windowed build clean.

**Deviations:** Baby Flame's parity drive asserts against the authored node union [60,77] rather than the coarse clip skill label [59,75] because `baby_flame_recover` overshoots to frames 76/77 (research finding #5, MEM057). T04 did not exist in the original S02 plan; it was added via `gsd_replan_slice` to capture the visual-readiness work surfaced during K001. The committed T03 plan included hop-aware bounce animation; the user directed its removal (bounce=VFX-only), resulting in a directed reversal, not an implementation bug.

**Known limitations:** No hurt/flinch reaction animation on damage targets (pre-existing; `anim_graph.ron` has no hurt node). Multi-hop bounce presentation is intentionally not wired in the windowed view (removed per user direction). Sprite layout is single-slot per side at scale 0.4; a 4-per-team multi-slot layout remains unimplemented.

## Verification

All four slice-level verification gates pass (re-confirmed at close):

1. `cargo test --test animation` → exit 0, 61 tests passed. Includes all S01 tests plus the four new S02 tests: baby_flame_impact_release_cue_resolves_to_in_range_impact_atlas_tile, baby_burner_launch_release_cue_resolves_to_in_range_impact_atlas_tile, baby_burner_player_frames_map_identity_within_heavy_attack_range, baby_flame_player_frames_map_identity_within_node_union. Impact-frame-on-rendered-frame invariant proven headless for Baby Flame (frame 70 ∈ skill [59,75], identity) and Baby Burner (all three barrier frames ∈ heavy_attack [23,45], identity).

2. `cargo build --features windowed` → exit 0, clean. No new warnings in render.rs or mod.rs.

3. `cargo test --features windowed` → exit 0, 25 test harnesses, 0 failed. render.rs unit tests cover: skill_start_node mapping per skill_id, same-skill cue hop advances without resetting player, auto-release-fallback narrowed to unbridged skills, release-bridge returns Released for bridged skills (not fallback), anim_clock tick accumulation, catch-up cap, non-positive fps, BEVYROGUE_ANIM_FPS parser, barrier_targets_only_the_casting_sprite.

4. `cargo test` (full headless suite) → exit 0, all harnesses clean, 0 failed. No windowed-gated dep leak (R002/R005 upheld).

Visual smoothness, damage-on-impact, and correct scale confirmed by user during K001 manual loop (idle no longer frantic; only casting sprite animates; Baby Flame cast→impact→recover linear; Baby Burner charge→launch→recovery sequential; scale 0.4 legible). Per slice plan, cargo winx was not launched from auto-mode.

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

No hurt/flinch reaction animation on the damage target (anim_graph.ron has no hurt node; pre-existing). Multi-hop bounce presentation intentionally removed (bounce=VFX-only). Sprite layout is single-slot at scale 0.4; 4-per-team multi-slot layout is deferred. VFX flash particles (S03).

## Follow-ups

None.

## Files Created/Modified

- `assets/data/digimon/agumon/skills.ron` — 
- `assets/digimon/agumon/anim_graph.ron` — 
- `tests/animation/atlas_binding.rs` — 
- `src/windowed/render.rs` — 
- `src/windowed/mod.rs` — 
- `src/combat/runtime/cue_barrier.rs` — 
