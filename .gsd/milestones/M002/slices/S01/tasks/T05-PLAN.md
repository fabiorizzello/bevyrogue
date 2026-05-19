---
estimated_steps: 1
estimated_files: 6
skills_used: []
---

# T05: Runtime AnimGraph player + RenderPlugin/UiPlugin split (windowed Agumon idle)

Create src/animation/player.rs as a FEATURE-AGNOSTIC FSM core (no #[cfg(feature)]): player state {current_node, elapsed_anim_frames}; a pure `advance(elapsed_anim_frames)` that derives the active sprite frame index from the node FrameRange honoring PlaybackModifier and `reverse`, and evaluates only TimeInNode/Always transitions. Add headless unit tests. Then split windowed.rs: introduce RenderPlugin (#[cfg(feature='windowed')] sprite system) and UiPlugin (egui panels). Wire both through windowed::register.

## Inputs

- None specified.

## Expected Output

- `cargo test passes headless (FSM core tests run; no winit/wgpu compiled)`
- `cargo build --features windowed compiles`
- `Documented soak run shows Agumon cycling idle with no panic`

## Verification

cargo test --test anim_player_fsm
