---
estimated_steps: 1
estimated_files: 18
skills_used: []
---

# T06: Run full S02 verification and close integration regressions

Expected executor skills: verify-before-complete, rust-testing, lint. Why: the prior tasks touch combat kernel, animation assets, Agumon data, feature-gated UI, and runtime composition; this final task proves the slice is actually integrated and catches regressions from the cross-boundary edits. Do: run the slice-level verification suite exactly, fix any failures in the smallest owning file, and only broaden scope if a failing test proves a real integration mismatch. Required commands: `cargo test --test timeline_two_clock_parity`; `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity`; `cargo test --test agumon_sharp_claws_asset`; `cargo test --test timeline_cue_barrier_pipeline`; `cargo test --features windowed --test windowed_preview_cache`; `cargo test --lib`; `cargo build --no-default-features`; `cargo build --features windowed`. If the environment has a display, also run `BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed` and capture whether it exited cleanly; if not display-capable, document that build and feature tests are the automated substitute. Failure Modes (Q5): asset hot-reload or feature-gated compilation can expose missing imports; use the failing command to localize. Load Profile (Q6): no stress benchmark required; the 10x concern for this slice is duplicate release or pending-state growth, covered by T04/T05 tests. Negative Tests (Q7): ensure all negative tests from earlier tasks remain in the committed test suite rather than relying on manual observation. Done when all mandatory commands pass and the final state truthfully supports the S02 demo claim.

## Inputs

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/animation/player.rs`
- `src/windowed.rs`
- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/widgets.rs`
- `src/ui/combat_panel/labels.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/data/digimon/agumon/skills.ron`
- `assets/data/digimon/agumon/unit.ron`
- `tests/timeline_two_clock_parity.rs`
- `tests/anim_player_fsm.rs`
- `tests/anim_graph_asset.rs`
- `tests/agumon_sharp_claws_asset.rs`
- `tests/timeline_cue_barrier_pipeline.rs`
- `tests/windowed_preview_cache.rs`

## Expected Output

- `src/combat/runtime/runner.rs`
- `src/combat/runtime/cue_barrier.rs`
- `src/combat/turn_system/pipeline/timeline_exec.rs`
- `src/animation/player.rs`
- `src/windowed.rs`
- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/widgets.rs`
- `src/ui/combat_panel/labels.rs`
- `assets/digimon/agumon/anim_graph.ron`
- `assets/data/digimon/agumon/skills.ron`
- `assets/data/digimon/agumon/unit.ron`
- `tests/timeline_two_clock_parity.rs`
- `tests/anim_player_fsm.rs`
- `tests/anim_graph_asset.rs`
- `tests/agumon_sharp_claws_asset.rs`
- `tests/timeline_cue_barrier_pipeline.rs`
- `tests/windowed_preview_cache.rs`

## Verification

cargo build --features windowed

## Observability Impact

Fresh verification evidence documents whether cue diagnostics, UI chip behavior, headless determinism, and windowed compilation all remain inspectable at slice closeout.
