---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T05: Wire windowed Sharp Claws playback and telegraph chip

Expected executor skills: bevy, frontend-design, make-interfaces-feel-better, rust-testing. Why: the slice demo is on-screen: Sharp Claws must visibly wind up, strike, release the kernel at the impact frame, recover, and show a telegraph chip so the player/future agent can see why damage is waiting. Do: in `src/windowed.rs`, opt the app into `TimelineClock(Clock::Windowed)` and replace the idle-only render system with a small presentation-aware state machine: use `StanceGraphRegistry` for idle, `SkillGraphRegistry` for `agumon_skill`, start Sharp Claws from the presentation anim/node emitted by T03/T04, trace frames, detect `FrameCueCommand::ReleaseKernel`, call the cue barrier release API, call `AnimGraphPlayer::fire_kernel_cue()`, and return to stance idle after recovery/Exit. Keep all Bevy window/render dependencies behind `#[cfg(feature = "windowed")]`. In `src/ui/combat_panel` add a compact telegraph chip/label for an awaiting Sharp Claws barrier (for example `Telegraph: Sharp Claws · impact pending`) and a helper that is unit-testable without rendering egui. Extend `tests/windowed_preview_cache.rs` or add a feature-gated windowed test for the telegraph helper/state labels. Ensure the Basic button/target flow uses the new `sharp_claws` data from T03 and still clears pending action on click. Failure Modes (Q5): missing skill graph handle falls back to idle and logs/diagnoses instead of releasing damage early; missing ReleaseKernel cue leaves the barrier visibly awaiting; duplicate frame reads do not duplicate release. Load Profile (Q6): one sprite/player state and one small UI label per frame; 10x FPS or repeated updates should not allocate unbounded state. Negative Tests (Q7): no release when graph handle missing, no duplicate release for the same cue frame, chip hides after release/resolution, and Basic preview still reports the Sharp Claws damage. Done when windowed build compiles, feature-gated tests pass, and the validation smoke can run for one second on a display-capable environment.

## Inputs

- `src/windowed.rs`
- `src/animation/registry.rs`
- `src/animation/player.rs`
- `src/animation/anim_graph.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/widgets.rs`
- `src/ui/combat_panel/labels.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/data/digimon/agumon/skills.ron`
- `tests/windowed_preview_cache.rs`

## Expected Output

- `src/windowed.rs`
- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/widgets.rs`
- `src/ui/combat_panel/labels.rs`
- `tests/windowed_preview_cache.rs`

## Verification

cargo test --features windowed --test windowed_preview_cache

## Observability Impact

Adds frame/cue traces plus a visible egui telegraph chip, giving future agents both log and UI surfaces for diagnosing stuck or prematurely released impact barriers.
