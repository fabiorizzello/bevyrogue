---
id: T02
parent: S01
milestone: M003
key_files:
  - tests/animation/atlas_binding.rs
key_decisions:
  - Resolve the impact frame from the loaded graph: scan sharp_claws* nodes for a FrameCueCommand::ReleaseKernel cue and compute clip frame as the inverse of render.rs local_frame_for (start()+at, honoring reverse), avoiding any hardcoded frame number
  - Drive the Sharp Claws sequence by firing fire_kernel_cue() once partway to satisfy the strike->recover KernelCue gate, since that transition is not time-driven
  - Source IDLE_RANGE [53,58] and ATTACK_RANGE [0,8] as named consts tied to clip.ron's authored ranges rather than magic literals in assertions
duration: 
verification_result: passed
completed_at: 2026-05-22T10:23:48.912Z
blocker_discovered: false
---

# T02: Added headless tests proving player-frame->atlas-index identity parity for idle + Sharp Claws and the impact-frame-on-rendered-frame invariant from the resolved ReleaseKernel cue

**Added headless tests proving player-frame->atlas-index identity parity for idle + Sharp Claws and the impact-frame-on-rendered-frame invariant from the resolved ReleaseKernel cue**

## What Happened

Extended tests/animation/atlas_binding.rs with three new lib-only parity/invariant tests, all driven against the real authored agumon assets via include_str! + ron::from_str (mirroring how render.rs loads graphs). (a) idle_player_frames_map_identity_within_idle_range: builds AnimGraphPlayer::new(graph.entry) on the parsed stance graph, advances 24 ticks (crossing the 6-frame Loop(count:0) wrap boundary), and asserts every advance_result().frame stays in [53,58] and that AtlasGeometry::atlas_index(frame)==Some(frame). (b) sharp_claws_player_frames_map_identity_within_attack_range: starts a player at NodeId("sharp_claws_windup"), advances windup(0,2)->strike(3,5)->recover(6,8), firing fire_kernel_cue() once at tick 6 to satisfy the strike->recover KernelCue gate, and asserts every frame stays within attack [0,8] and maps identity through atlas_index, breaking on exit. (c) sharp_claws_release_cue_resolves_to_in_range_impact_atlas_tile: resolves the ReleaseKernel cue from the loaded graph by scanning sharp_claws* nodes for FrameCueCommand::ReleaseKernel (no hardcoded frame), computes the impact clip frame via a clip_frame_at_cue helper that is the exact inverse of render.rs's local_frame_for (start()+at, honoring reverse), and asserts the resolved frame (3+1=4) lies in [0,8] and atlas_index(4)==Some(4). Added IDLE_RANGE/ATTACK_RANGE consts sourced from clip.ron's authored idle/attack ranges. Imports extended to AnimGraph, AnimGraphPlayer, AnimNode, FrameCueCommand, NodeId.

## Verification

Ran cargo test --test animation: 57 passed, 0 failed. The three new tests (idle_player_frames_map_identity_within_idle_range, sharp_claws_player_frames_map_identity_within_attack_range, sharp_claws_release_cue_resolves_to_in_range_impact_atlas_tile) all pass. The impact-frame test resolves the cue's `at` from the loaded graph (not a literal) and computes the clip frame via the documented inverse of local_frame_for.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | pass | 12000ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/animation/atlas_binding.rs`
