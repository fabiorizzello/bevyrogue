---
id: T02
parent: S02
milestone: M002
key_files:
  - src/animation/player.rs
  - tests/anim_player_fsm.rs
  - assets/digimon/agumon/anim_graph.ron
  - tests/anim_graph_asset.rs
key_decisions:
  - `AnimGraphPlayer` now latches `fire_kernel_cue()` as a one-shot predicate signal and consumes it when a `KernelCue` transition fires.
  - Agumon's shared skill graph now targets `clip: "all"` so Baby Flame can keep using `skill` frames while Sharp Claws uses `attack` frames in the same asset.
duration: 
verification_result: passed
completed_at: 2026-05-19T21:03:23.727Z
blocker_discovered: false
---

# T02: Added one-shot KernelCue transitions to `AnimGraphPlayer` and authored Agumon's Sharp Claws cue-gated animation nodes.

**Added one-shot KernelCue transitions to `AnimGraphPlayer` and authored Agumon's Sharp Claws cue-gated animation nodes.**

## What Happened

Extended `AnimGraphPlayer` with a feature-agnostic `fire_kernel_cue()` latch and taught `advance()` to evaluate `Predicate::KernelCue` alongside the existing `TimeInNode` and `Always` predicates. Added FSM coverage for blocked cue waits, one-shot cue consumption, and unknown-node fallback while preserving the earlier timing/modifier behavior. Updated Agumon's animation graph to keep Baby Flame as the entry path, add Sharp Claws windup/strike/recovery nodes over the atlas-backed `attack` frames, and author a `ReleaseKernel(())` cue on the strike node. Expanded asset parsing tests to assert the Sharp Claws nodes, cue frame, and malformed-RON failure behavior.

## Verification

Ran `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` and confirmed all 21 targeted tests passed, including the new KernelCue FSM cases, Sharp Claws asset parsing/cue assertions, gameplay-command rejection coverage, and atlas parity checks.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` | 0 | ✅ pass | 3479ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/animation/player.rs`
- `tests/anim_player_fsm.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/anim_graph_asset.rs`
