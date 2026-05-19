---
estimated_steps: 1
estimated_files: 4
skills_used: []
---

# T02: Teach AnimGraphPlayer KernelCue and author Sharp Claws cue graph

Expected executor skills: rust-best-practices, rust-testing, bevy. Why: the animation side must be able to block strike-to-recovery on a kernel cue and expose the exact ReleaseKernelCue frame for the runtime bridge. Do: extend `AnimGraphPlayer` with a feature-agnostic `fire_kernel_cue()` style API and predicate evaluation for `Predicate::KernelCue`; consume the cue when the transition fires so one release cannot satisfy multiple later gates. Add focused tests in `tests/anim_player_fsm.rs`: KernelCue transition does not fire before signal, fires after signal, consumes once, and TimeInNode/Always behavior remains intact. Update `assets/digimon/agumon/anim_graph.ron` to add Sharp Claws nodes using existing atlas-backed `attack` frames only: windup, strike with a `ReleaseKernel(())` frame cue, and recovery. Keep animation graph entry compatible with current Baby Flame users; callers may start a player at the Sharp Claws node by id. Update `tests/anim_graph_asset.rs` to assert the Sharp Claws nodes and ReleaseKernel cue parse. Failure Modes (Q5): malformed RON should fail parse tests; unknown node ids should preserve current safe fallback behavior. Load Profile (Q6): per animation tick remains O(transitions from current node + cues in current node), trivial for current graphs. Negative Tests (Q7): no transition before `fire_kernel_cue()`, no repeated transition from a stale cue, and `tests/anim_gameplay_command_forbidden.rs` must still reject gameplay commands in cues. Done when Sharp Claws animation data is pure presentation and the player can be tested headlessly.

## Inputs

- `src/animation/player.rs`
- `src/animation/anim_graph.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/digimon/agumon/clip.ron`
- `assets/digimon/agumon_atlas.json`
- `tests/anim_player_fsm.rs`
- `tests/anim_graph_asset.rs`
- `tests/anim_gameplay_command_forbidden.rs`
- `tests/clip_atlas_parity.rs`

## Expected Output

- `src/animation/player.rs`
- `tests/anim_player_fsm.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_graph_asset.rs`

## Verification

cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity

## Observability Impact

Player tests document the exact current_node/elapsed frame state before and after KernelCue release, making future stuck animation barriers diagnosable.
