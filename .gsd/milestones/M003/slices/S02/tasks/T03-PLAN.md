---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T03: Wire the Baby Burner and Baby Flame release bridges; auto-release only as fallback for unbridged skills

Why: With data authored (T01) and the seams generalized (T02), this task makes Baby Flame and Baby Burner actually release on the rendered impact frame, replacing the blanket auto-release at render.rs:264-277 (research build-order steps 3-4). Do: (1) Replace the lines-264-277 unconditional auto-release of non-SharpClaws barriers with a guard that auto-releases ONLY genuinely unbridged skills (skill_id not in {sharp_claws, baby_flame, agumon_ult}); keep the existing debug! diagnostic for that real fallback. (2) Generalize the pending-release block (currently gated on AgumonPlaybackMode::SharpClaws): for any Skill mode whose skill_id is bridged, when the current node carries a ReleaseKernel cue at the player's local_frame and it has not already been released for this (cue_id, node, local_frame, hop_index), call barrier.request_release(awaiting_cue_id); on Released|DuplicateRelease, call player.fire_kernel_cue() to advance the KernelCue-gated FSM transitions (baby_burner_charge->launch, baby_burner_launch->recovery, and the baby_flame_impact KernelCue self-loop that drives bounce hops) and record the ReleaseFrameKey. (3) Make the dedup guard hop-aware: include the barrier's hop_index in ReleaseFrameKey (or reset last_release_frame when awaiting_cue_id/hop_index changes) so repeated bounce_hop barriers (windowed bootstrap enables bouncing_fire) each release rather than being swallowed by the (cue_id,node,local_frame) key. (4) Confirm the multi-barrier walk for Baby Burner: windup barrier releases at end of baby_burner_charge (frame 30) -> fire advances to launch; impact barrier releases at baby_burner_launch ReleaseKernel (frame 32, damage lands) -> fire advances to recovery; recovery barrier releases at end of baby_burner_recovery (frame 45) -> timeline finishes; player exits to idle via the existing advance.exited path. (5) Add render.rs #[cfg(test)] unit tests proving a Baby Burner barrier and a Baby Flame barrier are released by the bridge at the cue frame (assert the release path returns CueReleaseResult::Released and the fallback debug! auto-release branch is NOT taken for these skill_ids), plus a hop-aware dedup test. Done-when: cargo test --test animation green (T01 invariants hold), full cargo test green (no headless regression, no windowed-gated dep leak per R002/R005), cargo build --features windowed green, and cargo test --features windowed runs the new render.rs release-bridge unit tests green. Smooth visual playback + damage-on-impact for skill/ultimate on both actors is deferred to manual K001 (do NOT launch cargo winx).

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/data/digimon/agumon/skills.ron`
- `src/combat/runtime/cue_barrier.rs`
- `src/animation/player.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo test --test animation 2>&1 | tail -15 && cargo build --features windowed 2>&1 | tail -10 && cargo test --features windowed 2>&1 | tail -20 && cargo test 2>&1 | tail -15

## Observability Impact

Narrow the auto-release debug! to genuinely unbridged skills; add a trace! at each multi-barrier FSM advance (released cue_id + node advanced into) under target windowed.agumon_playback.
