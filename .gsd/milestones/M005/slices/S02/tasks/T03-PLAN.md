---
estimated_steps: 9
estimated_files: 2
skills_used: []
---

# T03: Regression sweep + dep-gating closeout

Why: Lock the S02 contract and prove no headless/lib leak before closing the slice (R002/R005/R016). Mirrors the S01 T03 regression task and the verify-before-complete discipline: fresh evidence, no stale claims.

Do (verification only — no source changes):
1. `cargo test` (headless) — full lib/headless suite green; confirms the death/precedence mapping contract in `tests/animation/stance_reaction_mapping.rs` stays green and no headless regression.
2. `cargo test --features windowed` — windowed regression sweep + the new `render.rs` test-mod helpers (`is_death_reaction`, `fade_alpha`) all green.
3. `cargo build --features windowed` — windowed binary compiles with `drive_death_reactions` + `advance_death_fade` registered.
4. `cargo build` (default/headless) — confirms no windowed symbol leaked into the lib.
5. Dep-leak grep: `grep -nE "bevy::render|wgpu|winit|egui|bevy_render" src/windowed/render.rs` is expected to match only pre-existing windowed code (all S02 work is windowed by construction); confirm the only files changed by S01..S02 are `src/windowed/render.rs` (no lib crate edit) via `git diff --name-only` against the slice baseline. No new lib symbol was introduced, so R002/R005 hold.

Done-when: all four cargo commands exit 0, the grep/diff confirm the work is windowed-only and no lib file was modified, and the verdict records that K001 (visible death-frames-then-fade in `cargo winx`) remains a manual human sign-off that auto-mode does not perform. Skills: verify-before-complete, cargo-nextest.

Note (K001): the windowed binary is NEVER launched from auto-mode; visible death + off-field fade requires a manual `cargo winx` run by a human. This task stops at the build/test boundary and must not claim a visual PASS.

## Inputs

- `src/windowed/render.rs`
- `tests/animation/stance_reaction_mapping.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo test 2>&1 | tail -3 && cargo test --features windowed 2>&1 | tail -3 && cargo build --features windowed 2>&1 | tail -3 && cargo build 2>&1 | tail -3 && git diff --name-only
