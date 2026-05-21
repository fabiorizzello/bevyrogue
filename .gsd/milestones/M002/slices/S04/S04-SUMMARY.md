---
id: S04
parent: M002
milestone: M002
provides:
  - A reusable owner-neutral post-action reaction seam for future blueprint-side follow-up effects.
  - Deterministic Agumon Baby Burner detonate behavior with exact-once guards and adjacency filtering.
  - A feature-gated windowed flash projection pattern future combat cues can reuse without coupling presentation back into combat logic.
requires:
  - slice: S02
    provides: The deterministic cue-barrier/timeline contract and `UnitDied` payload semantics the post-action reaction seam depends on.
affects:
  - S05
  - S06
key_files:
  - src/combat/runtime/post_action.rs
  - src/combat/runtime/registry.rs
  - src/combat/runtime/mod.rs
  - src/combat/turn_system/pipeline/paths/single_target.rs
  - src/combat/blueprints/agumon/mod.rs
  - src/combat/blueprints/agumon/baby_burner.rs
  - src/ui/combat_panel/mod.rs
  - src/ui/combat_panel/labels.rs
  - src/ui/combat_panel/render.rs
  - src/windowed/mod.rs
  - tests/agumon_baby_burner_reactive.rs
  - tests/windowed_preview_cache.rs
  - tests/registry_internals.rs
key_decisions:
  - Keep the post-KO reaction seam owner-neutral by passing generic action context, roster snapshot data, and `UnitDied` payload into runtime registration rather than branching on Agumon inside shared combat code.
  - Project Agumon detonate observability from generic `OnKernelTransition::Blueprint` events into feature-gated deterministic UI state with a fixed frame lifetime instead of allowing presentation to drive combat logic.
patterns_established:
  - Owner-specific post-action reactions should hang off a generic runtime registry seam fed by committed combat context, not off Digimon-named branches in the turn pipeline.
  - Windowed-only combat cues should be projections from generic combat events into deterministic UI resources, preserving headless authority and keeping `CombatState` immutable.
observability_surfaces:
  - `UnitDied { heated_remaining }` payload for post-KO reaction context.
  - Detonate `OnDamageDealt` assertions in `tests/agumon_baby_burner_reactive.rs`.
  - `OnKernelTransition::Blueprint(owner="agumon", name="baby_burner_detonate", payload=UnitTarget)` events as the presentation seam.
  - `BabyBurnerFlashState` / combat-panel chip and tooltip in windowed mode for source, cast, signal, targets, and frame countdown.
drill_down_paths:
  - .gsd/milestones/M002/slices/S04/tasks/T01-SUMMARY.md
  - .gsd/milestones/M002/slices/S04/tasks/T02-SUMMARY.md
  - .gsd/milestones/M002/slices/S04/tasks/T03-SUMMARY.md
  - .gsd/milestones/M002/slices/S04/tasks/T04-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-21T06:41:40.878Z
blocker_discovered: false
---

# S04: Baby Burner reactive detonate + flash VFX

**Verified the Rust-only Baby Burner reactive detonate seam end-to-end: lethal Heated Baby Burner KOs now trigger deterministic adjacent detonate damage exactly once and feature-gated windowed flash projection without mutating combat state.**

## What Happened

S04 closed on top of four completed tasks spanning combat runtime, Agumon blueprint logic, and windowed presentation. T01 added an owner-neutral post-action reaction seam that carries primary-hit KO context plus stable roster snapshot data out of single-target resolution and through runtime registration, so blueprint owners can react after damage commits without adding Digimon-specific branching to shared combat code. T02 verified the existing Agumon Baby Burner registration against that seam: a lethal `agumon_ult` hit on a Heated primary target resolves deterministic adjacency through the shared targeting machinery, excludes the dead primary, ignores already-KO and non-adjacent units, applies the reactive detonate damage exactly once, and emits generic blueprint transition observability for each real detonation target. T03 confirmed the windowed-only presentation path is projection-only: generic `OnKernelTransition::Blueprint(owner="agumon", name="baby_burner_detonate", payload=UnitTarget)` events are folded into a fixed-frame `BabyBurnerFlashState` and combat-panel chip/tooltip surface, while HP and `CombatState` remain unchanged. T04 reran the slice regression/build matrix and documented the only remaining live-smoke gap as an execution-environment limitation inside `gsd_exec` rather than a repo regression. Together these tasks deliver the slice demo: Baby Burner reactive detonate plus visible flash proof, fully in Rust, deterministic, headless-safe, and ready for S05 full-kit assembly.

## Verification

Fresh closeout verification passed on current HEAD via `gsd_exec`: `cargo test --test agumon_baby_burner_reactive`, `cargo test --test unit_died_payload`, `cargo test --test timeline_cue_barrier_pipeline`, `cargo test --test timeline_two_clock_parity`, `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity`, `cargo test --features windowed --test windowed_preview_cache`, `cargo test --lib`, `cargo build --no-default-features`, and `cargo build --features windowed`. The reactive suite proved adjacency-only one-shot detonate behavior and negative cases; payload/timeline tests preserved the R002/R004 cue-barrier semantics; the windowed test proved fixed-frame flash projection and combat-state immutability; animation/atlas regressions stayed green, preserving R003; and both builds passed, preserving R005 headless/windowed dependency gating. Optional real-window smoke remains environment-limited inside the sandbox (`NoCompositor` / missing GPU) and is recorded as not proven rather than a product failure.

## Requirements Advanced

- R002 — Revalidated the headless-first cue/reaction flow through `unit_died_payload`, `timeline_cue_barrier_pipeline`, and the Baby Burner reactive integration tests.
- R004 — Revalidated deterministic suspension/resume and duplicate-release behavior with `timeline_cue_barrier_pipeline`, `timeline_two_clock_parity`, and exact-once detonate assertions.
- R005 — Confirmed the visible proof remains behind `feature = "windowed"` with `windowed_preview_cache` plus successful headless/windowed builds.

## Requirements Validated

- R002 — `cargo test --test unit_died_payload`, `cargo test --test timeline_cue_barrier_pipeline`, and `cargo test --test agumon_baby_burner_reactive` all passed on current HEAD.
- R003 — `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` passed, preserving animation/atlas parity and related gameplay seams.
- R004 — `cargo test --test timeline_cue_barrier_pipeline` and `cargo test --test timeline_two_clock_parity` passed, including duplicate-release/no-awaiting negative paths.
- R005 — `cargo test --features windowed --test windowed_preview_cache`, `cargo build --no-default-features`, and `cargo build --features windowed` all passed.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

Optional real-window smoke is not proven in the `gsd_exec` sandbox because Wayland lacks a compositor there and X11 fallback cannot find a GPU. The deterministic test/build matrix is therefore the canonical closeout evidence for this slice.

## Follow-ups

S05 should consume the same post-action reaction seam and generic blueprint transition observability when assembling the full Agumon-vs-dummy kit and any target hurt/blink polish.

## Files Created/Modified

- `src/combat/runtime/post_action.rs` — Defines the public post-action reaction seam and KO context passed to runtime registrations.
- `src/combat/runtime/registry.rs` — Registers and dispatches owner-neutral post-action reactions.
- `src/combat/runtime/mod.rs` — Exports the post-action runtime surface for blueprint consumers.
- `src/combat/turn_system/pipeline/paths/single_target.rs` — Threads primary-hit KO context through single-target resolution into the post-action seam.
- `src/combat/blueprints/agumon/mod.rs` — Hooks Agumon blueprint registration into the new runtime seam.
- `src/combat/blueprints/agumon/baby_burner.rs` — Implements deterministic Baby Burner detonate rules, adjacency filtering, and blueprint transition emission.
- `src/ui/combat_panel/mod.rs` — Introduces deterministic Baby Burner flash resource/state and transition observers.
- `src/ui/combat_panel/labels.rs` — Formats windowed flash chip labels and tooltip diagnostics.
- `src/ui/combat_panel/render.rs` — Renders the flash proof in the combat panel without mutating combat state.
- `src/windowed/mod.rs` — Registers the flash resource/systems in the windowed schedule.
- `tests/agumon_baby_burner_reactive.rs` — Covers lethal/non-lethal detonate behavior, adjacency rules, exact-once guards, and transition observability.
- `tests/windowed_preview_cache.rs` — Covers fixed-frame flash lifecycle, tooltip content, and combat-state immutability.
- `tests/registry_internals.rs` — Guards the runtime registry and post-action reaction surface.
