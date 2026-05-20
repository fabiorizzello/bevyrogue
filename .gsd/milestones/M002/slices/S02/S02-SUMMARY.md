---
id: S02
parent: M002
milestone: M002
provides:
  - A deterministic runtime barrier that downstream slices can reuse for visible multi-beat combat timing.
  - A data-backed Sharp Claws Basic attack path that later VFX, UI phase-strip, and full-kit work can build on.
  - A testable windowed telegraph proof surface for future presentation/debugging work.
requires:
  - slice: S01
    provides: Agumon graph/registry/schema groundwork, clip-atlas parity coverage, and the baseline windowed stance presentation path consumed by Sharp Claws.
affects:
  - S03
  - S04
  - S05
  - S06
key_files:
  - src/combat/runtime/runner.rs
  - src/combat/runtime/cue_barrier.rs
  - src/combat/runtime/mod.rs
  - src/combat/turn_system/pipeline/timeline_exec.rs
  - src/combat/turn_system/pipeline/mod.rs
  - src/animation/player.rs
  - src/windowed.rs
  - src/ui/combat_panel/mod.rs
  - src/ui/combat_panel/render.rs
  - src/ui/combat_panel/widgets.rs
  - src/ui/combat_panel/labels.rs
  - assets/digimon/agumon/anim_graph.ron
  - assets/data/digimon/agumon/skills.ron
  - assets/data/digimon/agumon/unit.ron
  - tests/timeline_two_clock_parity.rs
  - tests/anim_player_fsm.rs
  - tests/anim_graph_asset.rs
  - tests/agumon_sharp_claws_asset.rs
  - tests/timeline_cue_barrier_pipeline.rs
  - tests/windowed_preview_cache.rs
key_decisions:
  - Use a generic suspended-timeline cue barrier in combat runtime so windowed presentation can pause and later resume the same runner without windowed-only kernel code.
  - Keep Sharp Claws gameplay effects in skill timeline data and treat animation graphs as presentation-only, with a consumable KernelCue transition rather than gameplay commands in animation cues.
  - Expose telegraph-chip wording as pure helpers so windowed barrier diagnostics remain testable without egui rendering or a live display server.
patterns_established:
  - Two-clock execution pattern: headless auto runners may drain to completion, while windowed runners stop on presentation beats and resume only through an explicit cue-release API.
  - Animation-to-kernel handshake pattern: frame cue emits release, runtime unlatches barrier, AnimGraphPlayer consumes a one-shot kernel cue to unlock strike-to-recovery transition.
  - Windowed UI proof-surface pattern: put user-facing barrier labels/tooltips behind pure helpers and verify them with feature-gated tests instead of relying on interactive observation.
observability_surfaces:
  - Deterministic barrier state is now externally visible through windowed telegraph-chip labels/tooltips and through tests covering duplicate release, awaiting state, and post-release clearing.
  - Cue-barrier pipeline tests provide failure-localizing evidence for stuck barriers, duplicate releases, and queued-action rejection while resolving.
drill_down_paths:
  - .gsd/milestones/M002/slices/S02/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002/slices/S02/tasks/T02-SUMMARY.md
  - .gsd/milestones/M002/slices/S02/tasks/T03-SUMMARY.md
  - .gsd/milestones/M002/slices/S02/tasks/T04-SUMMARY.md
  - .gsd/milestones/M002/slices/S02/tasks/T05-SUMMARY.md
  - .gsd/milestones/M002/slices/S02/tasks/T06-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-20T16:26:23.754Z
blocker_discovered: false
---

# S02: Basic attack + two-clock impact barrier + telegraph chip

**Delivered Agumon Sharp Claws as a two-clock timeline/animation handshake: windowed execution now waits at an impact barrier, damage lands only after ReleaseKernelCue, and the UI exposes an awaiting-impact telegraph chip.**

## What Happened

S02 connected the first real gameplay-to-presentation handshake for M002. On the combat side, the runtime now distinguishes headless auto-advance from windowed cue barriers: windowed runners stop at presentation beats, suspended timeline state persists across frames, and duplicate or premature releases are harmless diagnostics/no-ops rather than duplicated damage. On the animation side, AnimGraphPlayer learned a consumable kernel-cue predicate so Sharp Claws can hold strike-to-recovery until the runtime releases the barrier. Agumon data was then rewired so Basic resolves a dedicated `sharp_claws` skill timeline carrying impact-beat damage plus the presentation cue metadata rather than reusing Baby Flame. Finally, the feature-gated windowed/UI surface was closed by exposing telegraph-chip helpers and feature-gated tests that prove Basic preview damage and awaiting/released barrier text without needing a live display. Fresh closeout verification on current HEAD passed for all mandatory slice commands; the only missing proof is the optional live one-second windowed smoke, which could not run here because the verification subprocess had no display/GPU-capable environment.

## Verification

Fresh S02 closeout evidence on current HEAD is green for all mandatory checks. `cargo test --test timeline_two_clock_parity` passed (2 tests). `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` passed (animation FSM, graph asset, gameplay-command guard, and clip/atlas parity all green). `cargo test --test agumon_sharp_claws_asset` passed (5 tests). `cargo test --test timeline_cue_barrier_pipeline` passed (5 tests, including duplicate-release and queued-action protection). `cargo test --features windowed --test windowed_preview_cache` passed (3 tests, including Sharp Claws preview damage and telegraph chip hide/show behavior). `cargo test --lib` passed. `cargo build --no-default-features` passed. `cargo build --features windowed` passed. For the optional live smoke, a current environment probe showed `DISPLAY`, `WAYLAND_DISPLAY`, and `XDG_SESSION_TYPE` unset in the verification lane, so `BEVYROGUE_VALIDATION_WINDOWED=1 BEVYROGUE_VALIDATION_WINDOWED_SOAK_SECS=1 cargo run --features windowed` was intentionally not re-run; the automated substitute is the passing windowed test/build coverage above.

## Requirements Advanced

- R002 — Proved headless-first cue logic by making windowed runs pause at presentation barriers while headless mode still drains to identical final intent streams.
- R004 — Added deterministic two-clock suspension/resume behavior with tests covering duplicate-release/no-awaiting no-ops and no damage before explicit release.
- R005 — Kept windowed playback, telegraph UI, and Bevy render concerns behind the feature-gated windowed surface while the combat barrier stays generic.
- R006 — Closed the slice with refreshed verification evidence on current HEAD, feature-gated tests, and explicit environment-limitation documentation.
- R003 — Preserved clip-atlas parity while reusing existing Agumon attack frames for Sharp Claws graph nodes.

## Requirements Validated

- R002 — `cargo test --test timeline_two_clock_parity` passed, proving headless auto and windowed manual cue release terminate with identical intent streams.
- R004 — `cargo test --test timeline_cue_barrier_pipeline` passed, proving damage is buffered until release and duplicate release does not duplicate effects.
- R005 — `cargo test --features windowed --test windowed_preview_cache` and `cargo build --features windowed` both passed, validating the feature-gated windowed surface.
- R003 — `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` passed, keeping graph/cue parsing and atlas parity green.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

Task summaries for T01-T04 were sparse, so the slice narrative was reconstructed from the slice plan plus fresh verification evidence. The optional live smoke was not executed in the verification lane because the current environment exposed no display session variables.

## Known Limitations

This closeout environment did not provide `DISPLAY`/`WAYLAND_DISPLAY`/GPU access, so live window creation was not re-validated here. Confidence for windowed behavior therefore comes from the passing `windowed_preview_cache` feature test and successful `--features windowed` build rather than an interactive render soak.

## Follow-ups

Update developer-facing validation docs/aliases to use the workspace-correct command `cargo run --features windowed --bin bevyrogue`, since bare `cargo run --features windowed` is a known multi-binary gotcha. Re-run the optional one-second visual smoke on a display/GPU-capable host before milestone closeout if interactive evidence is desired.

## Files Created/Modified

- `src/combat/runtime/runner.rs` — Made windowed runner surface AwaitingCue semantics needed for deterministic manual release.
- `src/combat/runtime/cue_barrier.rs` — Added suspended timeline barrier state and release/resume plumbing for two-clock execution.
- `src/combat/turn_system/pipeline/timeline_exec.rs` — Refactored timeline-backed action execution so queued intents apply only after explicit cue release.
- `src/animation/player.rs` — Added consumable KernelCue signaling to the animation player for Sharp Claws strike-to-recovery gating.
- `assets/digimon/agumon/anim_graph.ron` — Authored Sharp Claws windup/strike/recovery nodes using existing attack atlas frames and a ReleaseKernel cue.
- `assets/data/digimon/agumon/skills.ron` — Added Sharp Claws as the Agumon basic timeline with impact-beat barrier metadata and damage payload.
- `assets/data/digimon/agumon/unit.ron` — Repointed Agumon Basic to the new `sharp_claws` skill while keeping Baby Flame available.
- `src/ui/combat_panel/labels.rs` — Exposed pure telegraph-chip text/tooltip helpers for awaiting-impact diagnostics.
- `tests/windowed_preview_cache.rs` — Added feature-gated coverage for Sharp Claws preview damage and telegraph chip visibility/clearing.
