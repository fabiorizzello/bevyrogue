# S02: S02

**Goal:** Deliver Agumon's Sharp Claws basic attack as the first kernel-animation handshake: the animation graph plays windup to strike to recovery, the combat timeline stalls at the impact presentation barrier, damage is only committed after the animation emits ReleaseKernelCue, and the windowed UI exposes a telegraph chip while keeping headless/windowed intent streams deterministic and identical.
**Demo:** Sharp Claws windup to strike to recovery on screen; damage falls on the impact frame via ReleaseKernelCue; telegraph chip visible; I3 extended green.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Deterministic cue-awaiting runner contract exposed**
  - Files: `src/combat/runtime/runner.rs`, `tests/timeline_two_clock_parity.rs`
  - Verify: cargo test --test timeline_two_clock_parity

- [x] **T02: Teach AnimGraphPlayer KernelCue and author Sharp Claws cue graph**
  - Files: `src/animation/player.rs`, `tests/anim_player_fsm.rs`, `assets/digimon/agumon/anim_graph.ron`, `tests/anim_graph_asset.rs`
  - Verify: cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity

- [x] **T03: Route Agumon Basic through Sharp Claws timeline data**
  - Files: `assets/data/digimon/agumon/skills.ron`, `assets/data/digimon/agumon/unit.ron`, `tests/agumon_sharp_claws_asset.rs`
  - Verify: cargo test --test agumon_sharp_claws_asset --test anim_gameplay_command_forbidden

- [x] **T04: Persist suspended timelines and resume only after cue release**
  - Files: `src/combat/runtime/cue_barrier.rs`, `src/combat/runtime/mod.rs`, `src/combat/turn_system/pipeline/timeline_exec.rs`, `src/combat/turn_system/pipeline/mod.rs`, `tests/timeline_cue_barrier_pipeline.rs`
  - Verify: cargo test --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity --test compiled_timeline_runtime_dispatch

- [x] **T05: Wire windowed Sharp Claws playback and telegraph chip**
  - Files: `src/windowed.rs`, `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/render.rs`, `src/ui/combat_panel/widgets.rs`, `src/ui/combat_panel/labels.rs`, `tests/windowed_preview_cache.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache

- [x] **T06: Run full S02 verification and close integration regressions**
  - Files: `src/combat/runtime/runner.rs`, `src/combat/runtime/cue_barrier.rs`, `src/combat/turn_system/pipeline/timeline_exec.rs`, `src/animation/player.rs`, `src/windowed.rs`, `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/render.rs`, `src/ui/combat_panel/widgets.rs`, `src/ui/combat_panel/labels.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/data/digimon/agumon/skills.ron`, `assets/data/digimon/agumon/unit.ron`, `tests/timeline_two_clock_parity.rs`, `tests/anim_player_fsm.rs`, `tests/anim_graph_asset.rs`, `tests/agumon_sharp_claws_asset.rs`, `tests/timeline_cue_barrier_pipeline.rs`, `tests/windowed_preview_cache.rs`
  - Verify: cargo build --features windowed

## Files Likely Touched

- src/combat/runtime/runner.rs
- tests/timeline_two_clock_parity.rs
- src/animation/player.rs
- tests/anim_player_fsm.rs
- assets/digimon/agumon/anim_graph.ron
- tests/anim_graph_asset.rs
- assets/data/digimon/agumon/skills.ron
- assets/data/digimon/agumon/unit.ron
- tests/agumon_sharp_claws_asset.rs
- src/combat/runtime/cue_barrier.rs
- src/combat/runtime/mod.rs
- src/combat/turn_system/pipeline/timeline_exec.rs
- src/combat/turn_system/pipeline/mod.rs
- tests/timeline_cue_barrier_pipeline.rs
- src/windowed.rs
- src/ui/combat_panel/mod.rs
- src/ui/combat_panel/render.rs
- src/ui/combat_panel/widgets.rs
- src/ui/combat_panel/labels.rs
- tests/windowed_preview_cache.rs
