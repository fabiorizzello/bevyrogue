---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T02: Generalized the windowed Agumon cue bridge from a Sharp-Claws-only mode to a skill-parameterized, multi-barrier-aware Skill mode (pure refactor, Sharp Claws behavior unchanged).

Why: The current bridge hardcodes Sharp Claws: AgumonPlaybackMode is Idle | SharpClaws{cue_id, presentation_node}; sync_agumon_mode filters on SHARP_CLAWS_SKILL_ID and resets the player whenever the barrier cue_id changes. Baby Burner (3 barriers) and bouncing Baby Flame change the awaiting cue_id WITHIN one skill cast, so the player must keep advancing through the FSM rather than restart. This task is a pure refactor that preserves Sharp Claws behavior exactly (research build-order step 2). Do: (1) In src/windowed/mod.rs add constants: BABY_FLAME_SKILL_ID="baby_flame", AGUMON_ULT_SKILL_ID="agumon_ult", and node-name consts BABY_FLAME_CAST_NODE/BABY_FLAME_IMPACT_NODE, BABY_BURNER_CHARGE_NODE/BABY_BURNER_LAUNCH_NODE/BABY_BURNER_RECOVERY_NODE, matching anim_graph.ron. (2) In src/windowed/render.rs replace AgumonPlaybackMode::SharpClaws{..} with a generalized Skill variant carrying skill_id: String, awaiting_cue_id: String, and start_node: String (the FSM entry node for this skill). (3) Generalize the mode-sync: when an active barrier's skill_id matches the current Skill mode's skill_id, do NOT reset the player — only update awaiting_cue_id (and reset the dedup guard if the barrier's hop_index/cue changed). When the skill_id differs (or mode is Idle), start the skill: AnimGraphPlayer::new(start_node) where start_node is the skill's FSM entry (sharp_claws_windup for sharp_claws, baby_flame_cast for baby_flame, baby_burner_charge for agumon_ult), then set mode to Skill{skill_id, awaiting_cue_id, start_node}. Keep the InstantFallback resolution + missing-skill-graph diagnostics path intact. (4) Keep the lines-264-277 auto-release UNCHANGED for now (still auto-releases everything that is not sharp_claws) so Sharp Claws stays the only live bridge after this task — behavior is identical to S01. (5) Add/extend the existing render.rs #[cfg(test)] mod tests: a start-node mapping test per skill_id, and a sync test proving that a same-skill cue_id change updates awaiting_cue_id without resetting the player node. Done-when: cargo build --features windowed is green and cargo test --features windowed runs the render.rs unit tests green; Sharp Claws path is byte-for-byte behaviorally unchanged (the existing sharp_claws unit tests still pass).

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/animation/player.rs`
- `src/combat/runtime/cue_barrier.rs`

## Expected Output

- `src/windowed/render.rs`
- `src/windowed/mod.rs`

## Verification

cargo build --features windowed 2>&1 | tail -10 && cargo test --features windowed 2>&1 | tail -20

## Observability Impact

Generalize the per-tick trace! to log skill_id + awaiting_cue_id (instead of Sharp-Claws-specific fields); no new log lines, same target windowed.agumon_playback.
