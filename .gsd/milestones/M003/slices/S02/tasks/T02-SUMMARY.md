---
id: T02
parent: S02
milestone: M003
key_files:
  - src/windowed/render.rs
  - src/windowed/mod.rs
key_decisions:
  - Extracted the same-skill reconciliation into the pure fn classify_same_skill_sync (DifferentSkill|Unchanged|CueChanged) so the multi-barrier 'advance FSM without resetting player' decision is unit-testable — ResolvedAnimGraph's inner graph field is private to the lib crate, so a full sprite cannot be constructed in a bin-crate unit test.
  - Sharp Claws start_node is the FSM entry sharp_claws_windup (matching the old sharp_claws_start_node rewind), so dropping the presentation_node field is behavior-preserving; the strike presentation node is no longer needed to seed the player.
  - Left the lines 271-284 auto-release fallback unchanged (gates skill_id != sharp_claws) per task step 4, keeping Sharp Claws the only live bridge after this pure refactor.
  - Marked SHARP_CLAWS_STRIKE_NODE + BABY_FLAME_IMPACT_NODE + BABY_BURNER_LAUNCH/RECOVERY_NODE #[allow(dead_code)]: authored now as the bridged-node vocabulary, consumed when T03 activates the impact-release bridges.
duration: 
verification_result: passed
completed_at: 2026-05-22T10:57:52.759Z
blocker_discovered: false
---

# T02: Generalized the windowed Agumon cue bridge from a Sharp-Claws-only mode to a skill-parameterized, multi-barrier-aware Skill mode (pure refactor, Sharp Claws behavior unchanged).

**Generalized the windowed Agumon cue bridge from a Sharp-Claws-only mode to a skill-parameterized, multi-barrier-aware Skill mode (pure refactor, Sharp Claws behavior unchanged).**

## What Happened

Replaced AgumonPlaybackMode::SharpClaws{cue_id, presentation_node} with a generalized Skill{skill_id, awaiting_cue_id, start_node} variant in src/windowed/render.rs, and added the bridged-skill vocabulary to src/windowed/mod.rs (BABY_FLAME_SKILL_ID="baby_flame", AGUMON_ULT_SKILL_ID="agumon_ult", plus node consts BABY_FLAME_CAST/IMPACT, BABY_BURNER_CHARGE/LAUNCH/RECOVERY — verified against skills.ron SkillId() and anim_graph.ron node names).

The mode-sync was rewritten around two new pure seams: skill_start_node(skill_id) maps each bridged skill to its FSM entry node (sharp_claws->sharp_claws_windup, baby_flame->baby_flame_cast, agumon_ult->baby_burner_charge; unbridged -> None), and classify_same_skill_sync(mode, skill_id, cue_id) -> {DifferentSkill | Unchanged | CueChanged}. sync_agumon_mode now: returns early for unbridged skills (no start node); on a same-skill cue hop refreshes awaiting_cue_id and clears the last_release_frame dedup guard WITHOUT resetting the player (the key enabler for multi-barrier Baby Burner / bouncing Baby Flame); and only on DifferentSkill/Idle calls start_skill(skill_id, cue, start_node, graph) seeding AnimGraphPlayer::new(start_node). The InstantFallback resolution + missing-skill-graph diagnostics path was preserved exactly (warn + set last_missing_skill_graph_cue before start_skill, identical ordering to the old start_sharp_claws).

Behavior preservation for Sharp Claws is exact: its start node is still sharp_claws_windup (same node the old sharp_claws_start_node rewound to), the pending-release logic now matches Skill{awaiting_cue_id} and uses that cue (== the mode cue for the single-barrier Sharp Claws cast), and the lines 271-284 auto-release fallback (gates everything that is not sharp_claws while awaiting) was left UNCHANGED per the task — so Sharp Claws remains the only live bridge after this refactor. The per-tick trace! gained explicit skill_id + awaiting_cue_id fields (via mode_trace_fields) on the same windowed.agumon_playback target with no new log lines; a trace! was added on the in-place multi-barrier cue advance.

Removed the now-dead sharp_claws_start_node and sharp_claws_barrier helpers; SHARP_CLAWS_STRIKE_NODE plus the three not-yet-consumed presentation-node consts were marked #[allow(dead_code)] (they complete the bridged-node vocabulary T03 will consume).

## Verification

cargo build --features windowed: clean (exit 0, no warnings after annotating the forward-looking consts). cargo test --features windowed --bin bevyrogue: 7 passed including the 3 render.rs unit tests — the new skill_start_node_maps_each_bridged_skill_to_its_fsm_entry, the new same_skill_cue_hop_advances_without_resetting_player (covers Unchanged/CueChanged/DifferentSkill/Idle), and the retained duplicate_release_guard test. Full cargo test --features windowed: 23 harnesses report "test result: ok", 0 failures (Sharp Claws atlas-parity + windowed_only suites still green => byte-for-byte behavioral preservation). cargo clippy --features windowed --bin bevyrogue: only pre-existing warnings in untouched files (plugin.rs, registry.rs, frame_time.rs, agent_tracing.rs, clip.rs, turn_order_panel) — none in render.rs or mod.rs. Per K001, the windowed binary was NOT executed; visual smoothness is the user's manual sign-off.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 1120ms |
| 2 | `cargo test --features windowed --bin bevyrogue` | 0 | pass | 180ms |
| 3 | `cargo test --features windowed` | 0 | pass | 20000ms |
| 4 | `cargo clippy --features windowed --bin bevyrogue` | 0 | pass (only pre-existing warnings in untouched files) | 5080ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
