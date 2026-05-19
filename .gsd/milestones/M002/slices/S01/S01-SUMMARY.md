---
id: S01
milestone: M002
provides:
  - AnimGraphId required newtype on AnimGraph
  - FrameCue/FrameCueCommand/ReleaseKernelCue schema types
  - Predicate::KernelCue variant
  - GameplayCommandForbidden validation check
  - SkillGraphRegistry / StanceGraphRegistry (id->Handle resolution)
  - AnimationStancePaths / StanceGraphPaths routing
  - Agumon stance FSM asset (stance.ron) + whole-sheet clip range "all"
  - AnimGraphPlayer FSM core (feature-agnostic, headless-testable)
  - RenderPlugin / UiPlugin split in windowed.rs
key_files:
  - src/animation/anim_graph.rs
  - src/animation/player.rs
  - src/animation/registry.rs
  - src/animation/plugin.rs
  - src/animation/validation/
  - src/windowed.rs
  - assets/digimon/agumon/stance.ron
  - assets/digimon/agumon/clip.ron
  - tests/anim_graph_parse.rs
  - tests/anim_graph_asset.rs
  - tests/anim_validation.rs
  - tests/anim_gameplay_command_forbidden.rs
  - tests/anim_registry.rs
  - tests/anim_stance_asset.rs
  - tests/anim_player_fsm.rs
verification_result: passed
completed_at: 2026-05-19
---

# S01: Runtime player + sprite render + Stance FSM foundation

**All 5 tasks complete. Full cargo test green. cargo build --features windowed compiles.**

## What Was Built

### T01 — Schema extensions (previously completed)
Added `AnimGraphId`, `FrameCue`/`FrameCueCommand`/`ReleaseKernelCue`, and `Predicate::KernelCue` to the closed schema. 12/12 tests green.

### T02 — GameplayCommandForbidden validation (previously completed)
Added `AnimationValidationCheck::GameplayCommandForbidden` that errors when EmitDamage/EmitStatus/EmitHeal appear in node.on_enter or node.cues. Removed EmitDamage from production RON files.

### T03 — SkillGraphRegistry + StanceGraphRegistry
`src/animation/registry.rs` — two Resources wrapping `HashMap<AnimGraphId, Handle<AnimGraph>>` with pure `resolve()` map lookup. `populate_graph_registries` system routes loaded graphs to the correct registry by comparing asset paths to `SkillGraphPaths`/`StanceGraphPaths`. `has_matching_asset_event` helper moved here from plugin.rs to keep plugin.rs under the 500 LOC cap. 5/5 tests green.

### T04 — Agumon Stance FSM asset
Added `"all": (start: 0, end: 94)` to agumon clip.ron. Authored `assets/digimon/agumon/stance.ron` with idle(54-59, Loop∞), hurt(47-53), death(14-22), victory(78-94). Added `AnimationStancePaths` resource (in registry.rs) and wired it into `load_animation_graphs` so stance graphs route to `StanceGraphRegistry`. 3/3 tests green.

### T05 — AnimGraphPlayer FSM core + windowed split
`src/animation/player.rs` — feature-agnostic FSM: `advance(&mut self, graph) -> u32` evaluates TimeInNode/Always transitions (ignores KernelCue and future predicates), derives frame index honoring PlaybackModifier (Loop, Hold, SpeedMul) and `reverse`. 8/8 headless tests green.

`src/windowed.rs` split into `RenderPlugin` (Camera2d + advance_agumon_idle system) and `UiPlugin` (egui panels). `windowed::register` wires both. `cargo build --features windowed` compiles. Soak run: `advance_agumon_idle` traces `agumon_idle_frame` each tick cycling 54-59 via the infinite-loop idle stance node — no panic.

## Verification Evidence

| # | Command | Exit Code | Verdict |
|---|---------|-----------|---------|
| 1 | `cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation` | 0 | pass |
| 2 | `cargo test --test anim_gameplay_command_forbidden` | 0 | pass |
| 3 | `cargo test --test anim_registry` | 0 | pass (5 tests) |
| 4 | `cargo test --test anim_stance_asset` | 0 | pass (3 tests) |
| 5 | `cargo test --test anim_player_fsm` | 0 | pass (8 tests) |
| 6 | `cargo test` (full suite) | 0 | pass — all green |
| 7 | `cargo build --features windowed` | 0 | pass |

## Known Issues

None. S02 can plug in KernelCue-driven transitions and sprite rendering without schema rewrites.
