# S01: Runtime player + sprite render + Stance FSM foundation

**Goal:** Turn M001's animation data into the first on-screen behavior: a windowed Agumon sprite cycling idle driven by a data-authored Stance FSM through a feature-agnostic runtime AnimGraph player, with the schema seam closed-extended (id, cues, ReleaseKernelCue, Predicate::KernelCue) so S02-S07 plug in without rewrites.
**Demo:** cargo run --features windowed shows Agumon cycling idle via the stance graph (not hardcoded); M001 headless tests green; clip-atlas geometry parity test present and passing.

## Must-Haves

- cargo test fully green headless; cargo build --features windowed compiles; documented soak run shows Agumon cycling idle driven by stance graph with no panic.

## Proof Level

- This slice proves: integration

## Integration Closure

Upstream consumed: M001 AnimGraph/Clip schema + AnimationAssetPlugin + clip_geometry_parity. New wiring: RenderPlugin/UiPlugin split in windowed::register; AnimationStancePaths load path; SkillGraphRegistry/StanceGraphRegistry populated from loaded handles.

## Verification

- Run the task and slice verification checks for this slice.

## Tasks

- [ ] **T01: Closed-enum schema extensions + atomic id/asset/test migration** `est:3h`
  In src/animation/anim_graph.rs add a required `id: AnimGraphId` field to AnimGraph (closed transparent newtype). Add `cues: Vec<FrameCue>` with #[serde(default)] to AnimNode. Add `FrameCue { at: u32, command: FrameCueCommand }` where FrameCueCommand is a CLOSED enum carrying either a presentation Command or ReleaseKernelCue. Add ReleaseKernelCue. Add KernelCue variant to the closed Predicate enum. Update all test files and RON assets atomically.
  - Files: `src/animation/anim_graph.rs`, `src/animation/mod.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/digimon/renamon/anim_graph.ron`, `tests/anim_graph_asset.rs`, `tests/anim_graph_parse.rs`, `tests/anim_validation.rs`
  - Verify: cargo test --test anim_graph_parse --test anim_graph_asset --test anim_validation

- [ ] **T02: GameplayCommandForbidden validation check + anti-DRY test + EmitDamage remediation** `est:2h30m`
  In src/animation/validation/types.rs add AnimationValidationCheck::GameplayCommandForbidden and AnimationValidationReason::GameplayCommandInAnimGraph. In validation/graph.rs add check that EmitDamage/EmitStatus/EmitHeal in node.on_enter OR node.cues produces an Error diagnostic. Remove EmitDamage block from both production anim_graph.ron files. Add executable anti-DRY test asserting the live loaded agumon graph contains zero gameplay commands.
  - Files: `src/animation/validation/types.rs`, `src/animation/validation/command.rs`, `src/animation/validation/graph.rs`, `assets/digimon/agumon/anim_graph.ron`, `assets/digimon/renamon/anim_graph.ron`, `tests/anim_graph_asset.rs`, `tests/anim_validation.rs`, `tests/anim_gameplay_command_forbidden.rs`
  - Verify: cargo test --test anim_gameplay_command_forbidden --test anim_graph_asset --test anim_validation

- [ ] **T03: SkillGraphRegistry + StanceGraphRegistry (pure id->Handle resolution, R008)** `est:2h`
  Create src/animation/registry.rs with two Resource types each wrapping a map from AnimGraphId -> Handle<AnimGraph>, plus a pure `resolve(&self, id) -> Option<&Handle<AnimGraph>>` (no if-else dispatch; map lookup only). Classify by load-path provenance: skill graphs populate SkillGraphRegistry; stance graphs populate StanceGraphRegistry. Add a system that inserts entries keyed by the loaded AnimGraph.id once each handle resolves. Register both resources in AnimationAssetPlugin.
  - Files: `src/animation/registry.rs`, `src/animation/mod.rs`, `src/animation/plugin.rs`, `tests/anim_registry.rs`
  - Verify: cargo test --test anim_registry

- [ ] **T04: Agumon Stance FSM asset + whole-sheet clip range + stance load path** `est:2h`
  Add a whole-sheet named range to assets/digimon/agumon/clip.ron: 'all': (start: 0, end: 92). Author assets/digimon/agumon/stance.ron: id 'agumon_stance', clip 'all', entry 'idle', nodes idle(frames 53-58, modifier Loop), hurt(46-52), death(14-22), victory(76-92). Add DEFAULT_ANIM_STANCE_PATHS and AnimationStancePaths resource in plugin.rs; load + validate stance graphs through the existing AnimationAssetPlugin pipeline and feed them to StanceGraphRegistry.
  - Files: `assets/digimon/agumon/clip.ron`, `assets/digimon/agumon/stance.ron`, `src/animation/plugin.rs`, `tests/anim_stance_asset.rs`
  - Verify: cargo test --test anim_stance_asset --test clip_geometry_parity

- [ ] **T05: Runtime AnimGraph player + RenderPlugin/UiPlugin split (windowed Agumon idle)** `est:5h`
  Create src/animation/player.rs as a FEATURE-AGNOSTIC FSM core (no #[cfg(feature)]): player state {current_node, elapsed_anim_frames}; a pure `advance(elapsed_anim_frames)` that derives the active sprite frame index from the node FrameRange honoring PlaybackModifier and `reverse`, and evaluates only TimeInNode/Always transitions. Add headless unit tests. Then split windowed.rs: introduce RenderPlugin (#[cfg(feature='windowed')] sprite system) and UiPlugin (egui panels). Wire both through windowed::register.
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
