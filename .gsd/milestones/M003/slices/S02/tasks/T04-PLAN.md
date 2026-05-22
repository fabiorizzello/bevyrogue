---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Windowed visual-readiness fixes surfaced by manual K001 (animation clock, caster-gated barrier, sprite display scale)

Why: Manual K001 verification of the windowed bridge revealed three gaps blocking the S02 visual demo, none owned by an S01/S02 task. Do: (1) Add a fixed-rate AnimationClock resource in src/windowed/render.rs that accumulates Time::delta and advances the AnimGraphPlayer once per 1/fps instead of once per render frame; default 12 fps, override via BEVYROGUE_ANIM_FPS env; cap catch-up at 4 ticks to avoid spiral-of-death after a hitch; non-positive fps never ticks. Barrier release continues to sample the impact frame only on an animation tick. (2) Add `source: UnitId` to CueBarrierStatus in src/combat/runtime/cue_barrier.rs, populated from inflight.action.source (the real caster); add a pure predicate barrier_targets_sprite(status, unit_id) in render.rs and gate sync_agumon_mode + annotate_active_animation on it so only the casting sprite enters Skill mode (fixes the dummy animating an attack it did not launch). (3) Add SPRITE_DISPLAY_SCALE applied via Transform::with_scale at spawn (provisional 0.4 → ~205px, pending a multi-slot 4-per-team layout). Done-when: cargo build --features windowed green and cargo test --features windowed --bin bevyrogue green including the new unit tests (anim clock tick accumulation, catch-up cap, non-positive fps, env parser, barrier_targets_only_the_casting_sprite). Smooth playback / correct on-screen scale is manual K001 (do NOT launch cargo winx).

## Inputs

- `src/windowed/render.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/state.rs`
- `src/animation/player.rs`

## Expected Output

- `src/windowed/render.rs`
- `src/combat/runtime/cue_barrier.rs`

## Verification

cargo build --features windowed 2>&1 | tail -10 && cargo test --features windowed --bin bevyrogue 2>&1 | tail -20
