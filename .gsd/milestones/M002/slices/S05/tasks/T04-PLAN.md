---
estimated_steps: 3
estimated_files: 7
skills_used: []
---

# T04: Per-hop kernel cue: visible loop iterations = kernel hop_index

Why: Baby Flame's `BeatKind::Loop` body fires N hops via `BeatEvent { hop_index }`, but the AnimGraphPlayer today only knows `PlaybackModifier::Loop { count }`, and writing the kernel hop count into a `count: N` field on the animation graph would leak gameplay numbers into presentation (anti-DRY invariant guarded by `tests/anim_gameplay_command_forbidden.rs`). The runtime must drive each visible iteration via a kernel cue.

Do: (1) In `src/combat/runtime/runner.rs` + `src/combat/runtime/cue_barrier.rs`, extend the two-clock barrier so loop-body beats (`BeatKind::Loop` body iterations) also become presentation barriers when `Clock::Windowed`: after every loop body iteration completes its damage commit, suspend the runner with `AwaitingCue` carrying the current `hop_index`, mirroring the existing presentation-barrier semantics. `Clock::HeadlessAuto` continues to drain straight through. Duplicate-release/no-awaiting still no-op (R004 invariant). (2) Author a self-transition on the Baby Flame strike/impact node in `assets/digimon/agumon/anim_graph.ron` that consumes a `Predicate::KernelCue` and restarts the strike clip; the recovery node remains gated on the existing recovery cue. No gameplay numbers in graph data. (3) Wire windowed cue-release to fire one `ReleaseKernelCue` per loop hop (the existing release path is fine — the cue producer just needs to fire on the new barrier kind). (4) Add `tests/timeline_loop_hop_cue_parity.rs` asserting: headless-auto Baby Flame with bouncing-fire ON produces N hops and reaches Done; windowed Baby Flame with the same input suspends N times, requires N `resume_cue()` calls, and ends in the identical intent stream and final HP; releasing more cues than hops is a no-op. (5) Ensure `tests/timeline_two_clock_parity.rs`, `tests/timeline_cue_barrier_pipeline.rs`, `tests/anim_player_fsm.rs`, `tests/anim_graph_asset.rs`, `tests/anim_gameplay_command_forbidden.rs`, `tests/clip_atlas_parity.rs`, and `tests/bouncing_fire_off_baseline.rs` all stay green.

Done-when: the new test file passes and the listed regression tests stay green; `cargo build --features windowed` passes.

## Inputs

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/runtime/timeline.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/animation/player.rs`
- `src/animation/anim_graph.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/data/digimon/agumon/skills.ron`
- `tests/timeline_two_clock_parity.rs`
- `tests/timeline_cue_barrier_pipeline.rs`
- `tests/anim_gameplay_command_forbidden.rs`
- `tests/bouncing_fire_off_baseline.rs`

## Expected Output

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/runtime/mod.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/animation/player.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `tests/timeline_loop_hop_cue_parity.rs`

## Verification

cargo test --test timeline_loop_hop_cue_parity --test timeline_two_clock_parity --test timeline_cue_barrier_pipeline --test anim_gameplay_command_forbidden --test anim_player_fsm --test bouncing_fire_off_baseline --test clip_atlas_parity

## Observability Impact

Extends the suspended-timeline barrier surface: `AwaitingCue` now distinguishes single-impact vs per-hop barriers and the awaiting state exposes `hop_index`. Windowed telegraph chip diagnostics can label hop progression (e.g. `awaiting hop 2/3`) using existing label helpers; tests assert one cue release per hop.
