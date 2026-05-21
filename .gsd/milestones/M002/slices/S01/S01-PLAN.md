# S01: S01

**Goal:** Turn M001's animation data into the first on-screen behavior: a windowed Agumon sprite cycling idle driven by a data-authored Stance FSM through a feature-agnostic runtime AnimGraph player, with the schema seam closed-extended (id, cues, ReleaseKernelCue, Predicate::KernelCue) so S02-S07 plug in without rewrites.
**Demo:** cargo run --features windowed shows Agumon cycling idle via the stance graph (not hardcoded); M001 headless tests green; clip-atlas geometry parity test present and passing.

## Must-Haves

- Complete the planned slice outcomes.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [x] **T01: Closed-enum schema extensions (AnimGraphId, FrameCue, ReleaseKernelCue, KernelCue predicate) + atomic asset/test migration**
  - Files: `src/animation/anim_graph.rs`, `src/animation/mod.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/digimon/renamon/anim_graph.ron`, `tests/anim_graph_asset.rs`, `tests/anim_graph_parse.rs`, `tests/anim_validation.rs`
  - Verify: cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation

- [x] **T02: GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation**
  - Files: `src/animation/validation/types.rs`, `src/animation/validation/command.rs`, `src/animation/validation/graph.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/digimon/renamon/anim_graph.ron`, `tests/anim_graph_asset.rs`, `tests/anim_validation.rs`, `tests/anim_gameplay_command_forbidden.rs`
  - Verify: cargo test --test anim_gameplay_command_forbidden --test anim_graph_asset --test anim_validation

- [x] **T03: SkillGraphRegistry + StanceGraphRegistry (pure id->Handle resolution, R008)**
  - Files: `src/animation/registry.rs`, `src/animation/mod.rs`, `src/animation/plugin.rs`, `tests/anim_registry.rs`
  - Verify: cargo test --test anim_registry

- [x] **T04: Agumon Stance FSM asset + whole-sheet clip range + stance load path**
  - Files: `assets/digimon/agumon/clip.ron`, `assets/digimon/agumon/stance.ron`, `src/animation/plugin.rs`, `tests/anim_stance_asset.rs`
  - Verify: cargo test --test anim_stance_asset --test clip_geometry_parity

- [x] **T05: Runtime AnimGraph player + RenderPlugin/UiPlugin split (windowed Agumon idle)**
  - Files: `src/animation/player.rs`, `src/animation/mod.rs`, `src/animation/plugin.rs`, `src/windowed.rs`, `src/main.rs`, `tests/anim_player_fsm.rs`
  - Verify: cargo test --test anim_player_fsm

## Files Likely Touched

- src/animation/anim_graph.rs
- src/animation/mod.rs
- assets/digimon/agumon/anim_graph.ron
- assets/digimon/renamon/anim_graph.ron
- tests/anim_graph_asset.rs
- tests/anim_graph_parse.rs
- tests/anim_validation.rs
- src/animation/validation/types.rs
- src/animation/validation/command.rs
- src/animation/validation/graph.rs
- tests/anim_gameplay_command_forbidden.rs
- src/animation/registry.rs
- src/animation/plugin.rs
- tests/anim_registry.rs
- assets/digimon/agumon/clip.ron
- assets/digimon/agumon/stance.ron
- tests/anim_stance_asset.rs
- src/animation/player.rs
- src/windowed.rs
- src/main.rs
- tests/anim_player_fsm.rs
