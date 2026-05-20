---
estimated_steps: 10
estimated_files: 10
skills_used: []
---

# T04: Run S04 regression matrix and document live-smoke limits

---
estimated_steps: 5
estimated_files: 0
skills_used:
  - verify-before-complete
  - rust-testing
---
Why: S04 touches combat reactions, events, and feature-gated UI; closeout must prove it did not regress R002/R003/R004/R005.

Do: Run the full required matrix on current HEAD after T01-T03. If the environment lacks `DISPLAY`, `WAYLAND_DISPLAY`, or GPU access, do not attempt a live window; instead record that limitation in the task/slice summary and rely on the feature-gated windowed test plus `cargo build --features windowed`. Do not modify atlas/clip/animation assets unless a test failure proves it is necessary; MEM032/MEM037 require keeping clip-atlas parity green.

Done when: all mandatory commands in the slice Verification section pass, any optional live-smoke skip is explicitly justified by environment variables, and there are no unplanned asset changes.

## Inputs

- `tests/agumon_baby_burner_reactive.rs`
- `tests/unit_died_payload.rs`
- `tests/timeline_cue_barrier_pipeline.rs`
- `tests/timeline_two_clock_parity.rs`
- `tests/anim_player_fsm.rs`
- `tests/anim_graph_asset.rs`
- `tests/anim_gameplay_command_forbidden.rs`
- `tests/clip_atlas_parity.rs`
- `tests/windowed_preview_cache.rs`
- `Cargo.toml`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo test --test agumon_baby_burner_reactive --test unit_died_payload --test timeline_cue_barrier_pipeline --test timeline_two_clock_parity

## Observability Impact

Produces fresh closeout evidence for the new detonate signal and existing two-clock/windowed regression gates; no runtime observability changes.
