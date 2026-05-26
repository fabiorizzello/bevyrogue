---
id: T03
parent: S02
milestone: M005
key_files:
  - src/windowed/render.rs
  - tests/animation/stance_reaction_mapping.rs
key_decisions:
  - S02 dep-gating verified per-commit rather than against a single working-tree diff: the lib changes in src/animation/* are S01's (commit b1c3428), so the S02 windowed-only contract holds.
  - Did not launch the windowed binary (K001/R002); K001 visible death+fade left as a manual human sign-off, no visual PASS claimed.
duration: 
verification_result: passed
completed_at: 2026-05-26T08:44:14.677Z
blocker_discovered: false
---

# T03: Ran the S02 regression sweep and dep-gating closeout — all four cargo commands green, work confirmed windowed-only, no lib edit.

**Ran the S02 regression sweep and dep-gating closeout — all four cargo commands green, work confirmed windowed-only, no lib edit.**

## What Happened

Verification-only task; no source changes. Ran the four cargo commands from the plan: headless `cargo test` (exit 0), `cargo test --features windowed` (exit 0; lib unit tests 29 passed/0 failed including the new render.rs helpers `is_death_reaction_only_matches_unit_died` and `fade_alpha_lerps_full_to_zero`), `cargo build --features windowed` (exit 0; windowed binary with `drive_death_reactions` + `advance_death_fade` compiles), and default `cargo build` (exit 0; no windowed symbol leaked into the lib).

Dep-gating closeout (R002/R005): per-commit `git diff --name-only` shows the three S02 commits (0870c7a, d90296c, 34d5e85) each touched only `src/windowed/render.rs`. The lib death-mapping changes (`src/animation/mod.rs`, `src/animation/reaction.rs`, `tests/animation/stance_reaction_mapping.rs`) belong to the S01 commit b1c3428 and are consumed read-only by S02 — zero new lib symbols introduced by S02. `grep -nE "bevy::render|wgpu|winit|egui|bevy_render" src/windowed/render.rs` returned no matches, confirming no banned render/winit/egui leak.

K001 (visible death-frames-then-fade in `cargo winx`) remains a manual human sign-off. Per K001/R002 the windowed binary was NOT launched from auto-mode; this task stops at the build/test boundary and makes no visual PASS claim.

## Verification

All four cargo commands exited 0. Headless `cargo test` green. `cargo test --features windowed` green (lib: 29 passed, 0 failed). `cargo build --features windowed` and default `cargo build` both finished successfully. dep-gating: S02 commits touched only src/windowed/render.rs (per-commit git diff); grep for bevy::render|wgpu|winit|egui|bevy_render in render.rs found no matches; the new test helpers exist at render.rs:1038 (is_death_reaction) and :1115 (fade_alpha) with passing tests at :1822 and :1844.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test` | 0 | pass | 0ms |
| 2 | `cargo test --features windowed` | 0 | pass | 0ms |
| 3 | `cargo test --features windowed --lib (29 passed, 0 failed)` | 0 | pass | 0ms |
| 4 | `cargo build --features windowed` | 0 | pass | 0ms |
| 5 | `cargo build` | 0 | pass | 0ms |
| 6 | `grep -nE 'bevy::render|wgpu|winit|egui|bevy_render' src/windowed/render.rs` | 1 | pass (no matches — no dep leak) | 0ms |
| 7 | `git diff --name-only per S02 commit (0870c7a/d90296c/34d5e85)` | 0 | pass (only src/windowed/render.rs touched; no lib edit) | 0ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
- `tests/animation/stance_reaction_mapping.rs`
