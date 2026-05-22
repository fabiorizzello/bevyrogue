---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Prove player-frame->atlas-index parity for idle + Sharp Claws and the impact-frame-on-rendered-frame invariant (headless)

Why: the slice's headline guarantee is that the displayed atlas frame is the AnimGraphPlayer's current frame and that Sharp Claws damage lands on the rendered impact frame, not on keypress; this must be provable headless against the real authored assets. skills_used: tdd, verify-before-complete. Do: extend tests/animation/atlas_binding.rs. Parse the agumon stance graph (assets/digimon/agumon/stance.ron) and skill graph (assets/digimon/agumon/anim_graph.ron) via include_str! + ron::from_str into AnimGraph (mirror anim_player_fsm.rs). Build AtlasGeometry from the agumon ClipMeta. (a) Idle parity: construct AnimGraphPlayer::new(graph.entry) on the stance graph, advance several ticks, and assert every advance_result().frame falls in the idle clip range [53,58] and that AtlasGeometry::atlas_index(frame)==Some(frame) (1:1, in-range). (b) Sharp Claws parity: drive a player from the sharp_claws windup node through strike; assert each advance().frame stays within the attack clip range [0,8] and maps identity through atlas_index. (c) Impact-frame-on-rendered-frame invariant: locate the AnimNode whose cues contain FrameCueCommand::ReleaseKernel, compute the clip frame at that cue's local `at` using the node's FrameRange (start()+at, honoring reverse the same way local_frame_for does), and assert atlas_index(that_frame)==Some(that_frame) and that_frame lies within the attack range — i.e. the rendered atlas tile at the release tick is the impact frame. Keep all assertions lib-only (no windowed deps). Done when: cargo test --test animation passes with the three new parity/invariant tests, and the impact-frame test references the actual ReleaseKernel cue resolved from the loaded graph (not a hardcoded frame number).

## Inputs

- `src/animation/atlas.rs`
- `src/animation/player.rs`
- `src/animation/anim_graph.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/agumon/stance.ron`
- `assets/digimon/agumon/clip.ron`
- `tests/animation/atlas_binding.rs`

## Expected Output

- `tests/animation/atlas_binding.rs`

## Verification

cargo test --test animation
